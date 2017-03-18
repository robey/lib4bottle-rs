extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  // use std::io;
  use bytes::{Bytes};
  use futures::{Future, Stream, stream};
  use lib4bottle::bottle::{BottleType, framed_vec_stream, make_bottle};
  use lib4bottle::bottle_header::{Header};
  use lib4bottle::buffered_stream::{buffer_stream};
  use lib4bottle::stream_helpers::{drain_stream, make_stream_1, make_stream_4};
  use lib4bottle::to_hex::{FromHex, ToHex};
  use std::io;
  use std::iter;

  pub fn bytes123() -> Bytes {
    Bytes::from(vec![ 1, 2, 3 ])
  }


  #[test]
  fn write_a_small_frame() {
    let s = framed_vec_stream(make_stream_1(bytes123()));
    assert_eq!(
      s.collect().wait().unwrap().to_hex(),
      "0301020300"
    );
  }

  #[test]
  fn buffer_a_frame() {
    let s = make_stream_4(
      Bytes::from_static(b"he"),
      Bytes::from_static(b"ll"),
      Bytes::from_static(b"o sai"),
      Bytes::from_static(b"lor")
    );
    let b = framed_vec_stream(buffer_stream(s, 1024, true));
    assert_eq!(b.collect().wait().unwrap().to_hex(), "0c68656c6c6f207361696c6f7200");
  }

  #[test]
  fn write_power_of_2_frame() {
    for block_size in vec![ 128, 1024, 1 << 18, 1 << 21 ] {
      let mut buffer: Vec<u8> = Vec::with_capacity(block_size);
      buffer.resize(block_size, 0);
      let b = framed_vec_stream(make_stream_1(Bytes::from(buffer)));
      let out = drain_stream(b);
      assert_eq!(out.len(), block_size + 2);
      assert_eq!(out[0], (((block_size as f32).log(2.0) as u8) & 0x1f) + (0xf0 - 7));
      assert_eq!(out[out.len() - 1], 0);
    }
  }

  #[test]
  fn write_medium_frame() {
    // < 8K
    for block_size in vec![ 129, 1234, 8191 ] {
      let mut buffer: Vec<u8> = Vec::with_capacity(block_size);
      buffer.resize(block_size, 0);
      let b = framed_vec_stream(make_stream_1(Bytes::from(buffer)));
      let out = drain_stream(b);
      assert_eq!(out.len(), block_size + 3);
      assert_eq!(out[0], (block_size & 0x3f) as u8 + 0x80);
      assert_eq!(out[1], (block_size >> 6) as u8);
      assert_eq!(out[out.len() - 1], 0);
    }
  }

  #[test]
  fn write_large_frame() {
    // < 2M
    for block_size in vec![ 8193, 12345, 456123 ] {
      let mut buffer: Vec<u8> = Vec::with_capacity(block_size);
      buffer.resize(block_size, 0);
      let b = framed_vec_stream(make_stream_1(Bytes::from(buffer)));
      let out = drain_stream(b);
      assert_eq!(out.len(), block_size + 4);
      assert_eq!(out[0], (block_size & 0x1f) as u8 + 0xc0);
      assert_eq!(out[1], ((block_size >> 5) & 0xff) as u8);
      assert_eq!(out[2], (block_size >> 13) as u8);
      assert_eq!(out[out.len() - 1], 0);
    }
  }

  #[test]
  fn write_huge_frame() {
    // >= 2M
    for block_size in vec![ (1 << 21) + 1, 3998778 ] {
      let mut buffer: Vec<u8> = Vec::with_capacity(block_size);
      buffer.resize(block_size, 0);
      let b = framed_vec_stream(make_stream_1(Bytes::from(buffer)));
      let out = drain_stream(b);
      assert_eq!(out.len(), block_size + 5);
      assert_eq!(out[0], (block_size & 0xf) as u8 + 0xe0);
      assert_eq!(out[1], ((block_size >> 4) & 0xff) as u8);
      assert_eq!(out[2], ((block_size >> 12) & 0xff) as u8);
      assert_eq!(out[3], (block_size >> 20) as u8);
      assert_eq!(out[out.len() - 1], 0);
    }
  }

  #[test]
  fn write_a_small_bottle() {
    let mut h = Header::new();
    h.add_number(0, 150);
    let b = make_bottle(BottleType::Test, &h, iter::empty::<stream::Empty<Vec<Bytes>, io::Error>>());

    let magic_hex = "f09f8dbc0000";
    assert_eq!(b.collect().wait().unwrap().to_hex(), format!("{}a003800196ff", magic_hex));
  }

  #[test]
  fn write_a_small_data_bottle() {
    let data = make_stream_1(Bytes::from("ff00ff00".from_hex()));
    let b = make_bottle(BottleType::Test, &Header::new(), vec![ data ]);

    let magic_hex = "f09f8dbc0000";
    assert_eq!(b.collect().wait().unwrap().to_hex(), format!("{}a00004ff00ff0000ff", magic_hex));
  }

  #[test]
  fn write_a_nested_bottle() {
    let empty_stream = iter::empty::<stream::Empty<Vec<Bytes>, io::Error>>();
    let b1 = make_bottle(BottleType::Test, &Header::new(), empty_stream);
    let b2 = make_bottle(BottleType::Test2, &Header::new(), vec![ b1 ]);

    let magic_hex = "f09f8dbc0000";
    assert_eq!(b2.collect().wait().unwrap().to_hex(), format!("{}b00009{}a000ff00ff", magic_hex, magic_hex));
  }

  #[test]
  fn write_a_bottle_of_several_streams() {
    let data1 = make_stream_1(Bytes::from("f0f0f0".from_hex()));
    let data2 = make_stream_1(Bytes::from("e0e0e0".from_hex()));
    let data3 = make_stream_1(Bytes::from("cccccc".from_hex()));
    let b = make_bottle(BottleType::Test, &Header::new(), vec![ data1, data2, data3 ]);

    let magic_hex = "f09f8dbc0000";
    assert_eq!(b.collect().wait().unwrap().to_hex(), format!("{}a00003f0f0f00003e0e0e00003cccccc00ff", magic_hex));
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
