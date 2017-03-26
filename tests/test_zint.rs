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
  fn encode_length() {
    assert_eq!(zint::encode_length(1).to_hex(), "01");
    assert_eq!(zint::encode_length(100).to_hex(), "4064");
    assert_eq!(zint::encode_length(129).to_hex(), "4081");
    assert_eq!(zint::encode_length(127).to_hex(), "407f");
    assert_eq!(zint::encode_length(256).to_hex(), "4100");
    assert_eq!(zint::encode_length(1024).to_hex(), "4400");
    assert_eq!(zint::encode_length(12345).to_hex(), "7039");
    assert_eq!(zint::encode_length(3998778).to_hex(), "bd043a");
    assert_eq!(zint::encode_length(1 << 21).to_hex(), "a00000");
  }

  #[test]
  fn decode_first_length_byte() {
    assert_eq!(zint::decode_first_length_byte(0x01), ( 0, 1 ));
    assert_eq!(zint::decode_first_length_byte(0x0f), ( 0, 15 ));
    assert_eq!(zint::decode_first_length_byte(0x3f), ( 0, 63 ));
    assert_eq!(zint::decode_first_length_byte(0x40), ( 1, 0 ));
    assert_eq!(zint::decode_first_length_byte(0x4f), ( 1, 15 ));
    assert_eq!(zint::decode_first_length_byte(0x5f), ( 1, 31 ));
    assert_eq!(zint::decode_first_length_byte(0x80), ( 2, 0 ));
    assert_eq!(zint::decode_first_length_byte(0xbf), ( 2, 63 ));
  }

  #[test]
  fn decode_length() {
    assert_eq!(zint::decode_length(0, Bytes::from("01".from_hex()).as_ref()), 1);
    assert_eq!(zint::decode_length(1, Bytes::from("".from_hex()).as_ref()), 1);
    assert_eq!(zint::decode_length(15, Bytes::from("".from_hex()).as_ref()), 15);
    assert_eq!(zint::decode_length(0, Bytes::from("64".from_hex()).as_ref()), 100);
    assert_eq!(zint::decode_length(0, Bytes::from("81".from_hex()).as_ref()), 129);
    assert_eq!(zint::decode_length(0, Bytes::from("7f".from_hex()).as_ref()), 127);
    assert_eq!(zint::decode_length(1, Bytes::from("00".from_hex()).as_ref()), 256);
    assert_eq!(zint::decode_length(4, Bytes::from("00".from_hex()).as_ref()), 1024);
    assert_eq!(zint::decode_length(0x30, Bytes::from("39".from_hex()).as_ref()), 12345);
    assert_eq!(zint::decode_length(0x3d, Bytes::from("043a".from_hex()).as_ref()), 3998778);
    assert_eq!(zint::decode_length(0x20, Bytes::from("0000".from_hex()).as_ref()), 1 << 21);
  }
}
