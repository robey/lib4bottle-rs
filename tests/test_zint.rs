extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use std::io;
  use lib4bottle::hex::{FromHex, ToHex};
  use lib4bottle::zint;

  #[test]
  fn bytes_needed() {
    assert_eq!(zint::bytes_needed(1), 1);
    assert_eq!(zint::bytes_needed(254), 1);
    assert_eq!(zint::bytes_needed(255), 1);
    assert_eq!(zint::bytes_needed(256), 2);
    assert_eq!(zint::bytes_needed(1023), 2);
    assert_eq!(zint::bytes_needed(1024), 2);
    assert_eq!(zint::bytes_needed(16485), 2);
    assert_eq!(zint::bytes_needed(0xffffffff), 4);
    assert_eq!(zint::bytes_needed(0x100000000), 5);
    assert_eq!(zint::bytes_needed(0xff0010000000), 6);
    assert_eq!(zint::bytes_needed(0xff000010000000), 7);
    assert_eq!(zint::bytes_needed(0xff00000010000000), 8);
  }

  #[test]
  fn encode_packed_int() {
    assert_eq!(zint::encode_packed_int(0).to_hex(), "00");
    assert_eq!(zint::encode_packed_int(100).to_hex(), "64");
    assert_eq!(zint::encode_packed_int(129).to_hex(), "81");
    assert_eq!(zint::encode_packed_int(127).to_hex(), "7f");
    assert_eq!(zint::encode_packed_int(256).to_hex(), "0001");
    assert_eq!(zint::encode_packed_int(987654321).to_hex(), "b168de3a");
  }

  #[test]
  fn decode_packed_int() {
    assert_eq!(zint::decode_packed_int("00".from_hex().as_ref()).unwrap(), 0);
    assert_eq!(zint::decode_packed_int("0a".from_hex().as_ref()).unwrap(), 10);
    assert_eq!(zint::decode_packed_int("ff".from_hex().as_ref()).unwrap(), 255);
    assert_eq!(zint::decode_packed_int("64".from_hex().as_ref()).unwrap(), 100);
    assert_eq!(zint::decode_packed_int("81".from_hex().as_ref()).unwrap(), 129);
    assert_eq!(zint::decode_packed_int("7f".from_hex().as_ref()).unwrap(), 127);
    assert_eq!(zint::decode_packed_int("0001".from_hex().as_ref()).unwrap(), 256);
    assert_eq!(zint::decode_packed_int("b168de3a".from_hex().as_ref()).unwrap(), 987654321);
  }

  #[test]
  fn encode_length() {
    assert_eq!(zint::encode_length(1).to_hex(), "01");
    assert_eq!(zint::encode_length(100).to_hex(), "64");
    assert_eq!(zint::encode_length(129).to_hex(), "8102");
    assert_eq!(zint::encode_length(127).to_hex(), "7f");
    assert_eq!(zint::encode_length(256).to_hex(), "f1");
    assert_eq!(zint::encode_length(1024).to_hex(), "f3");
    assert_eq!(zint::encode_length(12345).to_hex(), "d98101");
    assert_eq!(zint::encode_length(3998778).to_hex(), "ea43d003");
    assert_eq!(zint::encode_length(87654321).to_hex(), "e1fb9753");
    assert_eq!(zint::encode_length(1 << 21).to_hex(), "fe");
  }

  #[test]
  fn encode_special_length() {
    assert_eq!(zint::encode_length(zint::END_OF_STREAM).to_hex(), "00");
    assert_eq!(zint::encode_length(zint::END_OF_ALL_STREAMS).to_hex(), "ff");
  }

  #[test]
  fn length_of_length() {
    assert_eq!(zint::length_of_length(0x00), 1);
    assert_eq!(zint::length_of_length(0x01), 1);
    assert_eq!(zint::length_of_length(0x64), 1);
    assert_eq!(zint::length_of_length(0x81), 2);
    assert_eq!(zint::length_of_length(0x7f), 1);
    assert_eq!(zint::length_of_length(0xf1), 1);
    assert_eq!(zint::length_of_length(0xf3), 1);
    assert_eq!(zint::length_of_length(0xd9), 3);
    assert_eq!(zint::length_of_length(0xea), 4);
    assert_eq!(zint::length_of_length(0xfe), 1);
    assert_eq!(zint::length_of_length(0xff), 1);
  }

  #[test]
  fn decode_length() {
    assert_eq!(zint::decode_length(&mut io::Cursor::new("00".from_hex())).unwrap(), 0);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("01".from_hex())).unwrap(), 1);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("64".from_hex())).unwrap(), 100);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("8102".from_hex())).unwrap(), 129);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("7f".from_hex())).unwrap(), 127);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("f1".from_hex())).unwrap(), 256);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("f3".from_hex())).unwrap(), 1024);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("d98101".from_hex())).unwrap(), 12345);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("ea43d003".from_hex())).unwrap(), 3998778);
    assert_eq!(zint::decode_length(&mut io::Cursor::new("fe".from_hex())).unwrap(), 1 << 21);
    assert_eq!(
      zint::decode_length(&mut io::Cursor::new("ff".from_hex())).unwrap(),
      zint::END_OF_ALL_STREAMS
    );
  }

  #[test]
  #[should_panic(expected = "UnexpectedEof")]
  fn decode_length_not_enough_bytes() {
    zint::decode_length(&mut io::Cursor::new("81".from_hex())).unwrap();
  }
}
