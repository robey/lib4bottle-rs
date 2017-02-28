extern crate lib4bottle;

mod test_zint {
  use std::io;
  use std::io::Seek;
  use lib4bottle::to_hex::{FromHex, ToHex};
  use lib4bottle::zint;

  #[test]
  fn encode_packed_int() {
    let mut cursor = io::Cursor::new(Vec::new());

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_packed_int(&mut cursor, 0).unwrap();
    assert_eq!(cursor.to_hex(), "00");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_packed_int(&mut cursor, 100).unwrap();
    assert_eq!(cursor.to_hex(), "64");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_packed_int(&mut cursor, 129).unwrap();
    assert_eq!(cursor.to_hex(), "81");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_packed_int(&mut cursor, 127).unwrap();
    assert_eq!(cursor.to_hex(), "7f");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_packed_int(&mut cursor, 256).unwrap();
    assert_eq!(cursor.to_hex(), "0001");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_packed_int(&mut cursor, 987654321).unwrap();
    assert_eq!(cursor.to_hex(), "b168de3a");
  }

  #[test]
  fn decode_packed_int() {
    assert_eq!(zint::decode_packed_int(&mut io::Cursor::new("00".from_hex())).unwrap(), 0);
    assert_eq!(zint::decode_packed_int(&mut io::Cursor::new("0a".from_hex())).unwrap(), 10);
    assert_eq!(zint::decode_packed_int(&mut io::Cursor::new("ff".from_hex())).unwrap(), 255);
    assert_eq!(zint::decode_packed_int(&mut io::Cursor::new("64".from_hex())).unwrap(), 100);
    assert_eq!(zint::decode_packed_int(&mut io::Cursor::new("81".from_hex())).unwrap(), 129);
    assert_eq!(zint::decode_packed_int(&mut io::Cursor::new("7f".from_hex())).unwrap(), 127);
    assert_eq!(zint::decode_packed_int(
      &mut io::Cursor::new("0001".from_hex())).unwrap(),
      256
    );
    assert_eq!(zint::decode_packed_int(
      &mut io::Cursor::new("b168de3a".from_hex())).unwrap(),
      987654321
    );
  }

  #[test]
  fn encode_length() {
    let mut cursor = io::Cursor::new(Vec::new());

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 1).unwrap();
    assert_eq!(cursor.to_hex(), "01");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 100).unwrap();
    assert_eq!(cursor.to_hex(), "64");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 129).unwrap();
    assert_eq!(cursor.to_hex(), "8102");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 127).unwrap();
    assert_eq!(cursor.to_hex(), "7f");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 256).unwrap();
    assert_eq!(cursor.to_hex(), "f1");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 1024).unwrap();
    assert_eq!(cursor.to_hex(), "f3");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 12345).unwrap();
    assert_eq!(cursor.to_hex(), "d98101");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 3998778).unwrap();
    assert_eq!(cursor.to_hex(), "ea43d003");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 87654321).unwrap();
    assert_eq!(cursor.to_hex(), "e1fb9753");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    zint::encode_length(&mut cursor, 1 << 21).unwrap();
    assert_eq!(cursor.to_hex(), "fe");
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
}


// "use strict";
//
// import * as zint from "../../lib/lib4bottle/zint";
//
// import "should";
// import "source-map-support/register";
//
// describe("zint", () => {
//
//
//   it("read length", () => {
//     zint.decodeLength(new Buffer("00", "hex")).should.eql(0);
//     zint.decodeLength(new Buffer("01", "hex")).should.eql(1);
//     zint.decodeLength(new Buffer("64", "hex")).should.eql(100);
//     zint.decodeLength(new Buffer("8102", "hex")).should.eql(129);
//     zint.decodeLength(new Buffer("7f", "hex")).should.eql(127);
//     zint.decodeLength(new Buffer("f1", "hex")).should.eql(256);
//     zint.decodeLength(new Buffer("f3", "hex")).should.eql(1024);
//     zint.decodeLength(new Buffer("d98101", "hex")).should.eql(12345);
//     zint.decodeLength(new Buffer("ea43d003", "hex")).should.eql(3998778);
//     zint.decodeLength(new Buffer("fe", "hex")).should.eql(Math.pow(2, 21));
//     zint.decodeLength(new Buffer("ff", "hex")).should.eql(-1);
//   });
// });
