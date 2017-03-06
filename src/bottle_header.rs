use std::error::Error;
use std::fmt;
use std::io;
use std::str;
use zint;

const KIND_BOOLEAN: u8 = 3;
const KIND_NUMBER: u8 = 2;
const KIND_STRING: u8 = 0;

enum FieldValue {
  Boolean,
  Number { value: u64 },
  String { value: String }
}

struct Field {
  id: u8,
  value: FieldValue,
}

pub struct Header {
  fields: Vec<Field>
}

impl Header {
  pub fn new() -> Header {
    Header { fields: Vec::new() }
  }

  pub fn add_bool(&mut self, id: u8) {
    assert!(id <= 15);
    self.fields.push(Field { id: id, value: FieldValue::Boolean });
  }

  pub fn add_number(&mut self, id: u8, value: u64) {
    assert!(id <= 15);
    self.fields.push(Field { id: id, value: FieldValue::Number { value: value } });
  }

  pub fn add_string(&mut self, id: u8, value: String) {
    assert!(id <= 15);
    self.fields.push(Field { id: id, value: FieldValue::String { value: value } });
  }

  pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
    for ref f in &self.fields {
      let content_length: usize = match f.value {
        FieldValue::Boolean => 0,
        FieldValue::Number { value } => zint::bytes_needed(value),
        FieldValue::String { ref value } => value.len()
      };
      let kind: u8 = match f.value {
        FieldValue::Boolean => KIND_BOOLEAN,
        FieldValue::Number { value: ref _value } => KIND_NUMBER,
        FieldValue::String { value: ref _value } => KIND_STRING
      };
      writer.write_all(&[
        (kind << 6) | (f.id << 2) | (((content_length >> 8) & 0x2) as u8),
        (content_length & 0xff) as u8
      ])?;

      // write content:
      match f.value {
        FieldValue::Boolean => (),
        FieldValue::Number { value } => zint::write_packed_int(writer, value)?,
        FieldValue::String { ref value } => writer.write_all(value.as_ref())?
      };
    }
    Ok(())
  }

  pub fn encode(&self) -> Vec<u8> {
    let mut cursor = io::Cursor::new(Vec::new());
    // unwrap is ok cuz it can' really fail
    self.write(&mut cursor).unwrap();
    cursor.into_inner()
  }

  pub fn decode(buffer: &[u8]) -> io::Result<Header> {
    let mut header = Header::new();
    let mut i: usize = 0;
    while i < buffer.len() {
      if i + 2 > buffer.len() { return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Truncated header")) }
      let kind = (buffer[i] & 0xc0) >> 6;
      let id = (buffer[i] & 0x3c) >> 2;
      let length: usize = (((buffer[i] & 0x3) as usize) << 8) + (buffer[i + 1] & 0xff) as usize;
      i += 2;
      if i + length > buffer.len() { return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Truncated header")) }

      let content = &buffer[i .. i + length];
      let value = match kind {
        KIND_BOOLEAN => FieldValue::Boolean,
        KIND_NUMBER => FieldValue::Number { value: zint::decode_packed_int(content.as_ref())? },
        KIND_STRING => FieldValue::String {
          value: str::from_utf8(content).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, e.description())
          })?.to_string()
        },
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown field kind"))
      };
      header.fields.push(Field { id: id, value: value });
      i += length;
    }
    Ok(header)
  }



//   export function unpackHeader(buffer) {
//   const header = new Header();
//   let i = 0;
//   while (i < buffer.length) {
//     if (i + 2 > buffer.length) throw new Error("Truncated header");
//     const type = (buffer[i] & 0xc0) >> 6;
//     const id = (buffer[i] & 0x3c) >> 2;
//     const length = (buffer[i] & 0x3) * 256 + (buffer[i + 1] & 0xff);
//     i += 2;
//     if (i + length > buffer.length) throw new Error("Truncated header");
//     const content = buffer.slice(i, i + length);
//     const field = { type, id };
//     switch (type) {
//       case TYPE_ZINT:
//         field.number = zint.decodePackedInt(content);
//         break;
//       case TYPE_STRING:
//         field.string = content.toString("UTF-8");
//         field.list = field.string.split("\x00");
//     }
//     header.fields.push(field);
//     i += length;
//   }
//   return header;
// }
}

impl fmt::Debug for Header {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Header({})", self.fields.iter().map(|f| match f.value {
      FieldValue::Boolean => format!("B{}", f.id),
      FieldValue::Number { ref value } => format!("N{}={}", f.id, value),
      FieldValue::String { ref value } => format!("S{}={:?}", f.id, value)
    }).collect::<Vec<String>>().join(", "))
  }
}
