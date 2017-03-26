extern crate bytes;
extern crate lib4bottle;

#[cfg(test)]
mod test_zint {
  use bytes::{Bytes};
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
  fn encode_packed_u64() {
    assert_eq!(zint::encode_packed_u64(0).to_hex(), "00");
    assert_eq!(zint::encode_packed_u64(100).to_hex(), "64");
    assert_eq!(zint::encode_packed_u64(129).to_hex(), "81");
    assert_eq!(zint::encode_packed_u64(127).to_hex(), "7f");
    assert_eq!(zint::encode_packed_u64(256).to_hex(), "0001");
    assert_eq!(zint::encode_packed_u64(987654321).to_hex(), "b168de3a");
  }

  #[test]
  fn decode_packed_u64() {
    assert_eq!(zint::decode_packed_u64(Bytes::from("00".from_hex())), 0);
    assert_eq!(zint::decode_packed_u64(Bytes::from("0a".from_hex())), 10);
    assert_eq!(zint::decode_packed_u64(Bytes::from("ff".from_hex())), 255);
    assert_eq!(zint::decode_packed_u64(Bytes::from("64".from_hex())), 100);
    assert_eq!(zint::decode_packed_u64(Bytes::from("81".from_hex())), 129);
    assert_eq!(zint::decode_packed_u64(Bytes::from("7f".from_hex())), 127);
    assert_eq!(zint::decode_packed_u64(Bytes::from("0001".from_hex())), 256);
    assert_eq!(zint::decode_packed_u64(Bytes::from("b168de3a".from_hex())), 987654321);
  }

  #[test]
  fn encode_u32() {
    assert_eq!(zint::encode_u32(1).to_hex(), "01000000");
    assert_eq!(zint::encode_u32(100).to_hex(), "64000000");
    assert_eq!(zint::encode_u32(129).to_hex(), "81000000");
    assert_eq!(zint::encode_u32(127).to_hex(), "7f000000");
    assert_eq!(zint::encode_u32(256).to_hex(), "00010000");
    assert_eq!(zint::encode_u32(1024).to_hex(), "00040000");
    assert_eq!(zint::encode_u32(12345).to_hex(), "39300000");
    assert_eq!(zint::encode_u32(3998778).to_hex(), "3a043d00");
    assert_eq!(zint::encode_u32(87654321).to_hex(), "b17f3905");
    assert_eq!(zint::encode_u32(1 << 21).to_hex(), "00002000");
  }

  #[test]
  fn encode_u32_special() {
    assert_eq!(zint::encode_u32(zint::END_OF_STREAM).to_hex(), "00000000");
    assert_eq!(zint::encode_u32(zint::END_OF_ALL_STREAMS).to_hex(), "ffffffff");
  }

  #[test]
  fn decode_u32() {
    assert_eq!(zint::decode_u32(Bytes::from("00000000".from_hex())), 0);
    assert_eq!(zint::decode_u32(Bytes::from("01000000".from_hex())), 1);
    assert_eq!(zint::decode_u32(Bytes::from("64000000".from_hex())), 100);
    assert_eq!(zint::decode_u32(Bytes::from("81000000".from_hex())), 129);
    assert_eq!(zint::decode_u32(Bytes::from("7f000000".from_hex())), 127);
    assert_eq!(zint::decode_u32(Bytes::from("00010000".from_hex())), 256);
    assert_eq!(zint::decode_u32(Bytes::from("00040000".from_hex())), 1024);
    assert_eq!(zint::decode_u32(Bytes::from("39300000".from_hex())), 12345);
    assert_eq!(zint::decode_u32(Bytes::from("3a043d00".from_hex())), 3998778);
    assert_eq!(zint::decode_u32(Bytes::from("00002000".from_hex())), 1 << 21);
    assert_eq!(
      zint::decode_u32(Bytes::from("ffffffff".from_hex())),
      zint::END_OF_ALL_STREAMS
    );
  }
}
