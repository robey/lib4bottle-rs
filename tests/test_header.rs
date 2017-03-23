// extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  // use std::io;
  // use bytes::{Bytes};
  use futures::{Future, Stream};
  use lib4bottle::header::{BottleType, Header};
  // use lib4bottle::buffered_stream::{buffer_stream};
  // use lib4bottle::stream_helpers::{drain_stream, make_stream_1, make_stream_4};
  use lib4bottle::hex::{ToHex};
  use lib4bottle::table::{Table};
  // use std::io;
  // use std::iter;

  // pub fn bytes123() -> Bytes {
  //   Bytes::from(vec![ 1, 2, 3 ])
  // }

  static MAGIC_HEX: &str = "f09f8dbc0000";

  #[test]
  fn write_header() {
    let mut t = Table::new();
    t.add_number(0, 150);
    let b = Header::new(BottleType::Test, t);
    assert_eq!(b.encode().collect().wait().unwrap().to_hex(), format!("{}a003800196", MAGIC_HEX));
  }
}




// const MAGIC_STRING = "f09f8dbc0000";
// const BASIC_MAGIC = MAGIC_STRING + "e000";
//
// describe("bottleReader", () => {
//   it("validates the header", future(() => {
//     const b = readBottle();
//     return new Promise(resolve => {
//       b.on("error", error => resolve(error));
//       sourceStream(new Buffer("00", "hex")).pipe(b);
//     }).then(error => {
//       error.message.should.match(/End of stream/);
//
//       const b2 = readBottle();
//       return new Promise(resolve => {
//         b2.on("error", error => resolve(error));
//         sourceStream(new Buffer("00ff00ff00ff00ff", "hex")).pipe(b2);
//       });
//     }).then(error => {
//       error.message.should.match(/magic/);
//
//       const b3 = readBottle();
//       return new Promise(resolve => {
//         b3.on("error", error => resolve(error));
//         sourceStream(new Buffer("f09f8dbcff000000", "hex")).pipe(b3);
//       });
//     }).then(error => {
//       error.message.should.match(/version/);
//
//       const b4 = readBottle();
//       return new Promise(resolve => {
//         b4.on("error", error => resolve(error));
//         sourceStream(new Buffer("f09f8dbc00ff0000", "hex")).pipe(b4);
//       });
//     }).then(error => {
//       error.message.should.match(/flags/);
//     });
//   }));
//
//   it("reads the header", future(() => {
//     const b = readBottle();
//     sourceStream(new Buffer("f09f8dbc0000c000", "hex")).pipe(b);
//     return b.readPromise().then(data => {
//       data.header.fields.length.should.eql(0);
//       data.type.should.eql(12);
//
//       const b2 = readBottle();
//       sourceStream(new Buffer("f09f8dbc0000e003800196", "hex")).pipe(b2);
//       return b2.readPromise();
//     }).then(data => {
//       data.header.fields.length.should.eql(1);
//       data.header.fields[0].number.should.eql(150);
//       data.type.should.eql(14);
//     });
//   }));
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
