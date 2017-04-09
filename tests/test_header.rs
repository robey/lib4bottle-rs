#![feature(conservative_impl_trait)]

extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_header {
  use futures::{Future, Stream};
  use lib4bottle::header::{BottleType, Header};
  use lib4bottle::stream_toolkit::{stream_of_hex, ToHex};
  use lib4bottle::table::Table;

  static MAGIC_HEX: &str = "f09f8dbc0000";

  #[test]
  fn write_header() {
    let mut t = Table::new();
    t.add_number(0, 150);
    let b = Header::new(BottleType::Test, t);
    assert_eq!(b.encode().collect().wait().unwrap().to_hex(), format!("{}a003800196", MAGIC_HEX));
  }

  #[test]
  #[should_panic(expected = "UnexpectedEof")]
  fn validate_header_length() {
    Header::decode(stream_of_hex("00")).wait().unwrap();
  }

  #[test]
  #[should_panic(expected = "Incorrect magic")]
  fn validate_header_magic() {
    Header::decode(stream_of_hex("00ff00ff00ff00ff")).wait().unwrap();
  }

  #[test]
  #[should_panic(expected = "Incompatible version")]
  fn validate_header_version() {
    Header::decode(stream_of_hex("f09f8dbcff000000")).wait().unwrap();
  }

  #[test]
  #[should_panic(expected = "Incompatible version")]
  fn validate_header_flags() {
    Header::decode(stream_of_hex("f09f8dbc00ff0000")).wait().unwrap();
  }

  #[test]
  #[should_panic(expected = "Unknown bottle type")]
  fn validate_header_bottle_type() {
    Header::decode(stream_of_hex("f09f8dbc0000f000")).wait().unwrap();
  }

  #[test]
  fn read_empty_header() {
    let s = stream_of_hex("f09f8dbc0000a000");
    let (h, s2) = Header::decode(s).wait().unwrap();
    assert_eq!(format!("{:?}", h), "Header(Test, Table())");
    // nothing left:
    assert_eq!(s2.into_stream().collect().wait().unwrap().to_hex(), "");
  }

  #[test]
  fn read_simple_header() {
    let s = stream_of_hex("f09f8dbc0000a003800196");
    let (h, s2) = Header::decode(s).wait().unwrap();
    assert_eq!(format!("{:?}", h), "Header(Test, Table(N0=150))");
    // nothing left:
    assert_eq!(s2.into_stream().collect().wait().unwrap().to_hex(), "");
  }

  #[test]
  fn read_sequentially() {
    let s = stream_of_hex("f09f8dbc0000a003800196f09f8dbc0000a003800196");
    let (h, s2) = Header::decode(s).wait().unwrap();
    assert_eq!(format!("{:?}", h), "Header(Test, Table(N0=150))");
    let (h2, s3) = Header::decode(s2).wait().unwrap();
    assert_eq!(format!("{:?}", h2), "Header(Test, Table(N0=150))");
    // nothing left:
    assert_eq!(s3.into_stream().collect().wait().unwrap().to_hex(), "");
  }
}






// const MAGIC_STRING = "f09f8dbc0000";
// const BASIC_MAGIC = MAGIC_STRING + "e000";
//
// describe("bottleReader", () => {
//
//   it("reads a data block", future(() => {
//     const b = readBottle();
//     sourceStream(new Buffer(`${BASIC_MAGIC}0568656c6c6f00ff`, "hex")).pipe(b);
//     return b.readPromise().then(() => {
//       return b.readPromise().then(dataStream => {
//         return pipeToBuffer(dataStream).then(data => {
//           data.toString().should.eql("hello");
//           return b.readPromise().then(dataStream => {
//             (dataStream == null).should.eql(true);
//           });
//         });
//       });
//     });
//   }));
//
//   it("reads a continuing data block", future(() => {
//     const b = readBottle();
//     sourceStream(new Buffer(`${BASIC_MAGIC}026865016c026c6f00ff`, "hex")).pipe(b);
//     return b.readPromise().then(() => {
//       return b.readPromise().then(dataStream => {
//         return pipeToBuffer(dataStream).then(data => {
//           data.toString().should.eql("hello");
//           return b.readPromise().then(data => {
//             (data == null).should.eql(true);
//           });
//         });
//       });
//     });
//   }));
//
//   it("reads several datas", future(() => {
//     const b = readBottle();
//     sourceStream(new Buffer(`${BASIC_MAGIC}03f0f0f00003e0e0e00003cccccc00ff`, "hex")).pipe(b);
//     return b.readPromise().then(() => {
//       return b.readPromise().then(dataStream => {
//         return pipeToBuffer(dataStream).then(data => {
//           data.toString("hex").should.eql("f0f0f0");
//           return b.readPromise();
//         });
//       }).then(dataStream => {
//         return pipeToBuffer(dataStream).then(data => {
//           data.toString("hex").should.eql("e0e0e0");
//           return b.readPromise();
//         });
//       }).then(dataStream => {
//         return pipeToBuffer(dataStream).then(data => {
//           data.toString("hex").should.eql("cccccc");
//           return b.readPromise();
//         });
//       }).then(dataStream => {
//         (dataStream == null).should.eql(true);
//       });
//     });
//   }));
//
//   it("reads several bottles from the same stream", future(() => {
//     const source = sourceStream(new Buffer(`${BASIC_MAGIC}0363617400ff${BASIC_MAGIC}0368617400ff`, "hex"));
//     const pull = new PullTransform({ transform: () => Promise.delay(10) });
//
//     const b1 = readBottle();
//     source.pipe(pull).subpipe(b1);
//     return b1.readPromise().then(() => {
//       return b1.readPromise().then(dataStream => {
//         return pipeToBuffer(dataStream).then(data => {
//           data.toString().should.eql("cat");
//           return b1.readPromise();
//         });
//       }).then(dataStream => {
//         (dataStream == null).should.eql(true);
//       });
//     }).then(() => {
//       const b2 = readBottle();
//       pull.subpipe(b2);
//       return b2.readPromise().then(() => {
//         return b2.readPromise().then(dataStream => {
//           return pipeToBuffer(dataStream).then(data => {
//             data.toString().should.eql("hat");
//             return b2.readPromise();
//           });
//         }).then(dataStream => {
//           (dataStream == null).should.eql(true);
//         });
//       });
//     }).then(() => {
//       const b3 = readBottle();
//       pull.subpipe(b3);
//       return b3.readPromise().then(() => {
//         throw new Error("expected end of stream");
//       }, error => {
//         error.message.should.match(/End of stream/);
//       });
//     });
//   }));
// });
