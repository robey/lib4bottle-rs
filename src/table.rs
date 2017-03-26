use bytes::{Bytes};
use std::error::Error;
use std::fmt;
use std::io;
use std::str;
use zint;

const KIND_BOOLEAN: u8 = 3;
const KIND_NUMBER: u8 = 2;
const KIND_STRING: u8 = 0;

/// An unordered set of TLV fields, where:
///   - type can be only boolean, unsigned int, or UTF-8 string
///   - length (of strings) can't exceed 1023 bytes
///   - key is a small int, from 0 - 15, per type
///
/// This is used to store metadata in the bottle header.
pub struct Table {
  fields: Vec<Field>
}

enum FieldValue {
  Boolean,
  Number(u64),
  String(String)
}

struct Field {
  id: u8,
  value: FieldValue,
}

impl Table {
  pub fn new() -> Table {
    Table { fields: Vec::new() }
  }

  /// Add a `true` boolean value. (False values are false by omission.)
  pub fn add_bool(&mut self, id: u8) {
    assert!(id <= 15);
    self.fields.push(Field { id: id, value: FieldValue::Boolean });
  }

  /// Add a u64 as a number.
  pub fn add_number(&mut self, id: u8, value: u64) {
    assert!(id <= 15);
    self.fields.push(Field { id: id, value: FieldValue::Number(value) });
  }

  /// Add a string.
  pub fn add_string(&mut self, id: u8, value: String) {
    assert!(id <= 15);
    self.fields.push(Field { id: id, value: FieldValue::String(value) });
  }

  pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
    for ref f in &self.fields {
      let content_length: usize = match f.value {
        FieldValue::Boolean => 0,
        FieldValue::Number(value) => zint::bytes_needed(value),
        FieldValue::String(ref value) => value.len()
      };
      let kind: u8 = match f.value {
        FieldValue::Boolean => KIND_BOOLEAN,
        FieldValue::Number(_) => KIND_NUMBER,
        FieldValue::String(_) => KIND_STRING
      };
      writer.write_all(&[
        (kind << 6) | (f.id << 2) | (((content_length >> 8) & 0x2) as u8),
        (content_length & 0xff) as u8
      ])?;

      // write content:
      match f.value {
        FieldValue::Boolean => (),
        FieldValue::Number(value) => writer.write_all(zint::encode_packed_u64(value).as_ref())?,
        FieldValue::String(ref value) => writer.write_all(value.as_ref())?
      };
    }
    Ok(())
  }

  pub fn encode(&self) -> Bytes {
    let mut cursor = io::Cursor::new(Vec::new());
    // unwrap is ok cuz it can't really fail
    self.write(&mut cursor).unwrap();
    Bytes::from(cursor.into_inner())
  }

  pub fn decode(buffer: Bytes) -> io::Result<Table> {
    let mut table = Table::new();
    let mut i: usize = 0;
    while i < buffer.len() {
      if i + 2 > buffer.len() { return Err(truncated_error()) }
      let kind = (buffer[i] & 0xc0) >> 6;
      let id = (buffer[i] & 0x3c) >> 2;
      let length: usize = (((buffer[i] & 0x3) as usize) << 8) + (buffer[i + 1] & 0xff) as usize;
      i += 2;
      if i + length > buffer.len() { return Err(truncated_error()) }

      let content = buffer.slice(i, i + length); //&buffer[i .. i + length];
      let value = match kind {
        KIND_BOOLEAN => FieldValue::Boolean,
        KIND_NUMBER => FieldValue::Number(zint::decode_packed_u64(content)),
        KIND_STRING => FieldValue::String(str::from_utf8(content.as_ref()).map_err(convert_error)?.to_string()),
        _ => return Err(unknown_kind_error())
      };
      table.fields.push(Field { id: id, value: value });
      i += length;
    }
    Ok(table)
  }
}

impl fmt::Debug for Table {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Table({})", self.fields.iter().map(|f| match f.value {
      FieldValue::Boolean => format!("B{}", f.id),
      FieldValue::Number(value) => format!("N{}={}", f.id, value),
      FieldValue::String(ref value) => format!("S{}={:?}", f.id, value)
    }).collect::<Vec<String>>().join(", "))
  }
}

// convert a UTF-8 decoding error into a normal I/O error
fn convert_error(e: str::Utf8Error) -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, e.description())
}

fn truncated_error() -> io::Error {
  io::Error::new(io::ErrorKind::UnexpectedEof, "Truncated header table")
}

fn unknown_kind_error() -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, "Unknown field kind")
}
