extern crate lib4bottle;

#[cfg(test)]
mod tests {
  // use std::io;
  // use std::io::Seek;
  use lib4bottle::to_hex::{FromHex, ToHex};
  use lib4bottle::bottle_header::Header;

  #[test]
  fn pack() {
    let mut m = Header::new();
    m.add_bool(1);
    assert_eq!(format!("{:?}", m), "Header(B1)");
    assert_eq!(m.encode().to_hex(), "c400");
    m.add_number(10, 1000);
    assert_eq!(format!("{:?}", m), "Header(B1, N10=1000)");
    assert_eq!(m.encode().to_hex(), "c400a802e803");
    m.add_string(3, String::from("iron"));
    assert_eq!(format!("{:?}", m), "Header(B1, N10=1000, S3=\"iron\")");
    assert_eq!(m.encode().to_hex(), "c400a802e8030c0469726f6e");
  }

  #[test]
  fn unpack() {
    assert_eq!(
      format!("{:?}", Header::decode("c400".from_hex().as_ref()).unwrap()),
      "Header(B1)"
    );
    assert_eq!(
      format!("{:?}", Header::decode("c400a802e803".from_hex().as_ref()).unwrap()),
      "Header(B1, N10=1000)"
    );
  }
}

// "use strict";
//
// import {
//   Header,
//   packHeader,
//   TYPE_BOOL,
//   TYPE_STRING,
//   TYPE_ZINT,
//   unpackHeader
// } from "../../lib/lib4bottle/bottle_header";
//
// import "should";
// import "source-map-support/register";
//
// describe("bottle_header", () => {
//   it("pack", () => {
//   });
//
//   it("unpack", () => {
//     unpackHeader(new Buffer("c400", "hex")).fields.should.eql([ { type: TYPE_BOOL, id: 1 } ]);
//     unpackHeader(new Buffer("c400a802e803", "hex")).fields.should.eql([
//       { type: TYPE_BOOL, id: 1 },
//       { type: TYPE_ZINT, id: 10, number: 1000 }
//     ]);
//     unpackHeader(new Buffer("c400a802e8030c0469726f6e", "hex")).fields.should.eql([
//       { type: TYPE_BOOL, id: 1 },
//       { type: TYPE_ZINT, id: 10, number: 1000 },
//       { type: TYPE_STRING, id: 3, list: [ "iron" ], string: "iron" }
//     ]);
//     unpackHeader(new Buffer("3c0d6f6e650074776f007468726565", "hex")).fields.should.eql([
//       { type: TYPE_STRING, id: 15, list: [ "one", "two", "three" ], string: "one\x00two\x00three" }
//     ]);
//   });
//
//   it("unpack truncated", () => {
//     (() => unpackHeader(new Buffer("c4", "hex"))).should.throw(/truncated/i);
//     (() => unpackHeader(new Buffer("c401", "hex"))).should.throw(/truncated/i);
//     (() => unpackHeader(new Buffer("c403ffff", "hex"))).should.throw(/truncated/i);
//   });
// });
