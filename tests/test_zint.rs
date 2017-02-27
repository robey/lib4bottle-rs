extern crate lib4bottle;

mod zint {
  use std::io;
  use std::io::Seek;
  use lib4bottle;
  use lib4bottle::ToHex;

  #[test]
  fn encode_packed_int() {
    let mut cursor = io::Cursor::new(Vec::new());

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    lib4bottle::encode_packed_int(&mut cursor, 10).unwrap();
    assert_eq!(cursor.to_hex(), "00");

    // let mut buffer = [ 0 as u8; 8 ];
    // let foo: &mut [u8] = &mut buffer;
    // lib4bottle::encode_packed_int(foo, 0);

    // let mut bytes = [0 as u8; 8];
    // let mut cursor = io::Cursor::new(&mut bytes as &mut [u8]);
    // // let mut cursor = io::Cursor::new(Vec::new());
    //
    // lib4bottle::encode_packed_int(&mut cursor, 10).unwrap();
    // let buffer = cursor.get_ref();
    // println!("cursor pos={} buffen len={}", cursor.position(), buffer.len());
    // println!("vec={:?}", buffer);
    // assert_eq!(lib4bottle::bytes_to_hex(&buffer[0..(cursor.position() as usize)]), "00");

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
//   it("encode packed", () => {
//     zint.encodePackedInt(0).toString("hex").should.eql("00");
//     zint.encodePackedInt(100).toString("hex").should.eql("64");
//     zint.encodePackedInt(129).toString("hex").should.eql("81");
//     zint.encodePackedInt(127).toString("hex").should.eql("7f");
//     zint.encodePackedInt(256).toString("hex").should.eql("0001");
//     zint.encodePackedInt(987654321).toString("hex").should.eql("b168de3a");
//   });
//
//   it("decode packed", () => {
//     zint.decodePackedInt(new Buffer("00", "hex")).should.eql(0);
//     zint.decodePackedInt(new Buffer("ff", "hex")).should.eql(255);
//     zint.decodePackedInt(new Buffer("64", "hex")).should.eql(100);
//     zint.decodePackedInt(new Buffer("81", "hex")).should.eql(129);
//     zint.decodePackedInt(new Buffer("7f", "hex")).should.eql(127);
//     zint.decodePackedInt(new Buffer("0001", "hex")).should.eql(256);
//     zint.decodePackedInt(new Buffer("b168de3a", "hex")).should.eql(987654321);
//   });
//
//   it("encode length", () => {
//     zint.encodeLength(1).toString("hex").should.eql("01");
//     zint.encodeLength(100).toString("hex").should.eql("64");
//     zint.encodeLength(129).toString("hex").should.eql("8102");
//     zint.encodeLength(127).toString("hex").should.eql("7f");
//     zint.encodeLength(256).toString("hex").should.eql("f1");
//     zint.encodeLength(1024).toString("hex").should.eql("f3");
//     zint.encodeLength(12345).toString("hex").should.eql("d98101");
//     zint.encodeLength(3998778).toString("hex").should.eql("ea43d003");
//     zint.encodeLength(87654321).toString("hex").should.eql("e1fb9753");
//     zint.encodeLength(Math.pow(2, 21)).toString("hex").should.eql("fe");
//   });
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
