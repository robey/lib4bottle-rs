use std::io;

pub trait ToHex {
  fn to_hex(&self) -> String;
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
