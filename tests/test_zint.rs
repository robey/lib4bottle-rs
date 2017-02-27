extern crate lib4bottle;

mod zint {
  use std::io;
  use std::io::Seek;
  use lib4bottle;
  use lib4bottle::{FromHex, ToHex};

  #[test]
  fn encode_packed_int() {
    let mut cursor = io::Cursor::new(Vec::new());

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 0).unwrap();
    assert_eq!(cursor.to_hex(), "00");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 100).unwrap();
    assert_eq!(cursor.to_hex(), "64");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 129).unwrap();
    assert_eq!(cursor.to_hex(), "81");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 127).unwrap();
    assert_eq!(cursor.to_hex(), "7f");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 256).unwrap();
    assert_eq!(cursor.to_hex(), "0001");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 987654321).unwrap();
    assert_eq!(cursor.to_hex(), "b168de3a");
  }

  #[test]
  fn decode_packed_int() {
    assert_eq!(lib4bottle::decode_packed_int(&mut io::Cursor::new("00".from_hex())).unwrap(), 0);
    assert_eq!(lib4bottle::decode_packed_int(&mut io::Cursor::new("0a".from_hex())).unwrap(), 10);
    assert_eq!(lib4bottle::decode_packed_int(&mut io::Cursor::new("ff".from_hex())).unwrap(), 255);
    assert_eq!(lib4bottle::decode_packed_int(&mut io::Cursor::new("64".from_hex())).unwrap(), 100);
    assert_eq!(lib4bottle::decode_packed_int(&mut io::Cursor::new("81".from_hex())).unwrap(), 129);
    assert_eq!(lib4bottle::decode_packed_int(&mut io::Cursor::new("7f".from_hex())).unwrap(), 127);
    assert_eq!(lib4bottle::decode_packed_int(
      &mut io::Cursor::new("0001".from_hex())).unwrap(),
      256
    );
    assert_eq!(lib4bottle::decode_packed_int(
      &mut io::Cursor::new("b168de3a".from_hex())).unwrap(),
      987654321
    );
  }

  #[test]
  fn encode_length() {
    let mut cursor = io::Cursor::new(Vec::new());

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 1).unwrap();
    assert_eq!(cursor.to_hex(), "01");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 100).unwrap();
    assert_eq!(cursor.to_hex(), "64");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 129).unwrap();
    assert_eq!(cursor.to_hex(), "8102");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 127).unwrap();
    assert_eq!(cursor.to_hex(), "7f");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 256).unwrap();
    assert_eq!(cursor.to_hex(), "f1");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 1024).unwrap();
    assert_eq!(cursor.to_hex(), "f3");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 12345).unwrap();
    assert_eq!(cursor.to_hex(), "d98101");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 3998778).unwrap();
    assert_eq!(cursor.to_hex(), "ea43d003");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 87654321).unwrap();
    assert_eq!(cursor.to_hex(), "e1fb9753");

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_length(&mut cursor, 1 << 21).unwrap();
    assert_eq!(cursor.to_hex(), "fe");
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
//   it("determine length of length", () => {
//     zint.lengthLength(0x00).should.eql(1);
//     zint.lengthLength(0x01).should.eql(1);
//     zint.lengthLength(0x64).should.eql(1);
//     zint.lengthLength(0x81).should.eql(2);
//     zint.lengthLength(0x7f).should.eql(1);
//     zint.lengthLength(0xf1).should.eql(1);
//     zint.lengthLength(0xf3).should.eql(1);
//     zint.lengthLength(0xd9).should.eql(3);
//     zint.lengthLength(0xea).should.eql(4);
//     zint.lengthLength(0xfe).should.eql(1);
//     zint.lengthLength(0xff).should.eql(1);
//   });
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
