extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_bottle {
  use bytes::{Bytes};
  use futures::{Future, Stream, stream};
  use lib4bottle::bottle::{Bottle, frame_stream};
  use lib4bottle::header::{BottleType, Header};
  use lib4bottle::stream_helpers::{stream_of, stream_of_streams, stream_of_vec};
  use lib4bottle::stream_reader::{ByteFrame};
  use lib4bottle::hex::{FromHex, ToHex};
  use lib4bottle::table::{Table};
  use std::io;

  static MAGIC_HEX: &str = "f09f8dbc0000";

  #[test]
  fn write_a_small_frame() {
    let s = frame_stream(stream_of(Bytes::from("010203".from_hex())));
    assert_eq!(
      s.collect().wait().unwrap().to_hex(),
      "0301020300"
    );
  }

  #[test]
  fn buffer_a_frame() {
    let s = stream_of_vec(vec![
      Bytes::from_static(b"he"),
      Bytes::from_static(b"ll"),
      Bytes::from_static(b"o sai"),
      Bytes::from_static(b"lor")
    ]);
    let b = frame_stream(s);
    assert_eq!(b.collect().wait().unwrap().to_hex(), "0c68656c6c6f207361696c6f7200");
  }

  #[test]
  fn write_a_small_bottle() {
    let mut t = Table::new();
    t.add_number(0, 150);
    let b = Bottle::new(BottleType::Test, t, stream::empty::<stream::Empty<Bytes, io::Error>, io::Error>());
    assert_eq!(b.encode().collect().wait().unwrap().to_hex(), format!("{}a003800196ff", MAGIC_HEX));
  }

  #[test]
  fn write_a_small_data_bottle() {
    let data = stream_of(Bytes::from("ff00ff00".from_hex()));
    let b = Bottle::new(BottleType::Test, Table::new(), stream_of_streams(vec![ data ]));
    assert_eq!(b.encode().collect().wait().unwrap().to_hex(), format!("{}a00004ff00ff0000ff", MAGIC_HEX));
  }

  #[test]
  fn write_a_nested_bottle() {
    let empty_stream = stream::empty::<stream::Empty<Bytes, io::Error>, io::Error>();
    let b1 = Bottle::new(BottleType::Test, Table::new(), empty_stream);
    let b2 = Bottle::new(BottleType::Test2, Table::new(), stream_of_streams(vec![ b1.encode() ]));
    assert_eq!(
      b2.encode().collect().wait().unwrap().to_hex(),
      format!("{}b00009{}a000ff00ff", MAGIC_HEX, MAGIC_HEX)
    );
  }

  #[test]
  fn write_a_bottle_of_several_streams() {
    let data1 = stream_of(Bytes::from("f0f0f0".from_hex()));
    let data2 = stream_of(Bytes::from("e0e0e0".from_hex()));
    let data3 = stream_of(Bytes::from("cccccc".from_hex()));
    let b = Bottle::new(BottleType::Test, Table::new(), stream_of_streams(vec![ data1, data2, data3 ]));
    assert_eq!(
      b.encode().collect().wait().unwrap().to_hex(),
      format!("{}a00003f0f0f00003e0e0e00003cccccc00ff", MAGIC_HEX)
    );
  }
}

// // const MAGIC_STRING = "f09f8dbc0000";
// // const BASIC_MAGIC = MAGIC_STRING + "e000";
// //
// // describe("bottleReader", () => {

// //   it("reads a data block", future(() => {
// //     const b = readBottle();
// //     sourceStream(new Buffer(`${BASIC_MAGIC}0568656c6c6f00ff`, "hex")).pipe(b);
// //     return b.readPromise().then(() => {
// //       return b.readPromise().then(dataStream => {
// //         return pipeToBuffer(dataStream).then(data => {
// //           data.toString().should.eql("hello");
// //           return b.readPromise().then(dataStream => {
// //             (dataStream == null).should.eql(true);
// //           });
// //         });
// //       });
// //     });
// //   }));
// //
// //   it("reads a continuing data block", future(() => {
// //     const b = readBottle();
// //     sourceStream(new Buffer(`${BASIC_MAGIC}026865016c026c6f00ff`, "hex")).pipe(b);
// //     return b.readPromise().then(() => {
// //       return b.readPromise().then(dataStream => {
// //         return pipeToBuffer(dataStream).then(data => {
// //           data.toString().should.eql("hello");
// //           return b.readPromise().then(data => {
// //             (data == null).should.eql(true);
// //           });
// //         });
// //       });
// //     });
// //   }));
// //
// //   it("reads several datas", future(() => {
// //     const b = readBottle();
// //     sourceStream(new Buffer(`${BASIC_MAGIC}03f0f0f00003e0e0e00003cccccc00ff`, "hex")).pipe(b);
// //     return b.readPromise().then(() => {
// //       return b.readPromise().then(dataStream => {
// //         return pipeToBuffer(dataStream).then(data => {
// //           data.toString("hex").should.eql("f0f0f0");
// //           return b.readPromise();
// //         });
// //       }).then(dataStream => {
// //         return pipeToBuffer(dataStream).then(data => {
// //           data.toString("hex").should.eql("e0e0e0");
// //           return b.readPromise();
// //         });
// //       }).then(dataStream => {
// //         return pipeToBuffer(dataStream).then(data => {
// //           data.toString("hex").should.eql("cccccc");
// //           return b.readPromise();
// //         });
// //       }).then(dataStream => {
// //         (dataStream == null).should.eql(true);
// //       });
// //     });
// //   }));
// //
// //   it("reads several bottles from the same stream", future(() => {
// //     const source = sourceStream(new Buffer(`${BASIC_MAGIC}0363617400ff${BASIC_MAGIC}0368617400ff`, "hex"));
// //     const pull = new PullTransform({ transform: () => Promise.delay(10) });
// //
// //     const b1 = readBottle();
// //     source.pipe(pull).subpipe(b1);
// //     return b1.readPromise().then(() => {
// //       return b1.readPromise().then(dataStream => {
// //         return pipeToBuffer(dataStream).then(data => {
// //           data.toString().should.eql("cat");
// //           return b1.readPromise();
// //         });
// //       }).then(dataStream => {
// //         (dataStream == null).should.eql(true);
// //       });
// //     }).then(() => {
// //       const b2 = readBottle();
// //       pull.subpipe(b2);
// //       return b2.readPromise().then(() => {
// //         return b2.readPromise().then(dataStream => {
// //           return pipeToBuffer(dataStream).then(data => {
// //             data.toString().should.eql("hat");
// //             return b2.readPromise();
// //           });
// //         }).then(dataStream => {
// //           (dataStream == null).should.eql(true);
// //         });
// //       });
// //     }).then(() => {
// //       const b3 = readBottle();
// //       pull.subpipe(b3);
// //       return b3.readPromise().then(() => {
// //         throw new Error("expected end of stream");
// //       }, error => {
// //         error.message.should.match(/End of stream/);
// //       });
// //     });
// //   }));
// // });
