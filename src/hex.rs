use bytes::Bytes;
use std::io;

pub trait ToHex {
  fn to_hex(&self) -> String;
}

pub trait FromHex {
  fn from_hex(&self) -> Vec<u8>;
}

impl ToHex for [u8] {
  fn to_hex(&self) -> String {
    self.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("")
  }
}

impl ToHex for io::Cursor<Vec<u8>> {
  fn to_hex(&self) -> String {
    let slice = self.get_ref();
    slice[0..(self.position() as usize)].to_hex()
  }
}

impl ToHex for Bytes {
  fn to_hex(&self) -> String {
    self.as_ref().to_hex()
  }
}

impl<T> ToHex for Vec<T> where T: ToHex {
  fn to_hex(&self) -> String {
    self.iter().map(|item| item.to_hex()).collect::<Vec<String>>().join("")
  }
}

impl<'a> FromHex for &'a str {
  fn from_hex(&self) -> Vec<u8> {
    // rust still doesn't have step_by! :(
    (0 .. self.len() / 2).map(|i| {
      u8::from_str_radix(&self[i * 2 .. (i + 1) * 2], 16).unwrap()
    }).collect::<Vec<u8>>()
  }
}
