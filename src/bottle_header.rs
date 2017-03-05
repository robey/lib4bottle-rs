use std::io;
use std::fmt;
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
      let content: Vec<u8> = match f.value {
        FieldValue::Boolean => Vec::new(),
        FieldValue::Number { value } => zint::encode_packed_int(value),
        FieldValue::String { ref value } => value.clone().into_bytes()
      };
      let kind: u8 = match f.value {
        FieldValue::Boolean => KIND_BOOLEAN,
        FieldValue::Number { value: ref _value } => KIND_NUMBER,
        FieldValue::String { value: ref _value } => KIND_STRING
      };
      writer.write_all(&[
        (kind << 6) | (f.id << 2) | (((content.len() >> 8) & 0x2) as u8),
        (content.len() & 0xff) as u8
      ])?;
      writer.write_all(content.as_ref())?;
    }
    Ok(())
  }

  pub fn encode(&self) -> Vec<u8> {
    let mut cursor = io::Cursor::new(Vec::new());
    // unwrap is ok cuz it can' really fail
    self.write(&mut cursor).unwrap();
    cursor.into_inner()
  }
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
