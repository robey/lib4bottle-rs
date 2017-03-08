extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  // use std::io;
  use futures::{Future, Stream, stream};
  use lib4bottle::bottle::framed_stream;
  use lib4bottle::to_hex::{FromHex, ToHex};
  use bytes::Bytes;

  #[test]
  fn write_a_small_frame() {
    let buffer = Bytes::from(vec![ 1, 2, 3 ]);
    let s = framed_stream(stream::iter(vec![ Ok(buffer) ]));
    // - collect the stream into a future(vec)
    // - wait for the future to be a result(vec)
    // - for each vec item: pull out the bytes and hexify
    // - collect the strings back into a vec
    // - join to one string
    assert_eq!(
      s.collect().wait().unwrap().to_hex(),
      "03010203"
    );
  }
}



//
// describe("framingStream", () => {
//   it("writes a small frame", future(() => {
//     const fs = framingStream();
//     const p = pipeToBuffer(fs);
//     fs.write(new Buffer([ 1, 2, 3 ]));
//     fs.end();
//     return p.then(data => {
//       data.toString("hex").should.eql("0301020300");
//     });
//   }));
//
//   it("buffers up a frame", future(() => {
//     const bs = bufferStream();
//     const fs = framingStream();
//     bs.pipe(fs);
//     const p = pipeToBuffer(fs);
//     bs.write(new Buffer("he"));
//     bs.write(new Buffer("ll"));
//     bs.write(new Buffer("o sai"));
//     bs.write(new Buffer("lor"));
//     bs.end();
//     return p.then(data => {
//       data.toString("hex").should.eql("0c68656c6c6f207361696c6f7200");
//     });
//   }));
//
//   it("writes a power-of-two frame", future(() => {
//     return Promise.all([ 128, 1024, Math.pow(2, 18), Math.pow(2, 21) ].map(blockSize => {
//       const fs = framingStream();
//       const p = pipeToBuffer(fs);
//       const b = new Buffer(blockSize);
//       b.fill(0);
//       fs.write(b);
//       fs.end();
//       return p.then(data => {
//         data.length.should.eql(blockSize + 2);
//         data[0].should.eql((Math.log(blockSize) / Math.log(2)) + 0xf0 - 7);
//       });
//     }));
//   }));
//
//   it("writes a medium (< 8K) frame", future(() => {
//     return Promise.all([ 129, 1234, 8191 ].map(blockSize => {
//       const fs = framingStream();
//       const p = pipeToBuffer(fs);
//       const b = new Buffer(blockSize);
//       b.fill(0);
//       fs.write(b);
//       fs.end();
//       return p.then(data => {
//         data.length.should.eql(blockSize + 3);
//         data[0].should.eql((blockSize & 0x3f) + 0x80);
//         data[1].should.eql(blockSize >> 6);
//       });
//     }));
//   }));
//
//   it("writes a large (< 2M) frame", future(() => {
//     return Promise.all([ 8193, 12345, 456123 ].map(blockSize => {
//       const fs = framingStream();
//       const p = pipeToBuffer(fs);
//       const b = new Buffer(blockSize);
//       b.fill(0);
//       fs.write(b);
//       fs.end();
//       return p.then(data => {
//         data.length.should.eql(blockSize + 4);
//         data[0].should.eql((blockSize & 0x1f) + 0xc0);
//         data[1].should.eql((blockSize >> 5) & 0xff);
//         data[2].should.eql((blockSize >> 13));
//       });
//     }));
//   }));
//
//   it("writes a huge (>= 2M) frame", future(() => {
//     return Promise.all([ Math.pow(2, 21) + 1, 3998778 ].map(blockSize => {
//       const fs = framingStream();
//       const p = pipeToBuffer(fs);
//       const b = new Buffer(blockSize);
//       b.fill(0);
//       fs.write(b);
//       fs.end();
//       return p.then(data => {
//         data.length.should.eql(blockSize + 5);
//         data[0].should.eql((blockSize & 0xf) + 0xe0);
//         data[1].should.eql((blockSize >> 4) & 0xff);
//         data[2].should.eql((blockSize >> 12) & 0xff);
//         data[3].should.eql((blockSize >> 20) & 0xff);
//       });
//     }));
//   }));
// });




// import Promise from "bluebird";
// import stream from "stream";
// import { bufferStream, pipeToBuffer, PullTransform, sourceStream } from "stream-toolkit";
// import { future } from "mocha-sprinkles";
// import { Header } from "../../lib/lib4bottle/bottle_header";
// import { readBottle, writeBottle } from "../../lib/lib4bottle/bottle_stream";
//
// import "should";
// import "source-map-support/register";
//
// const MAGIC_STRING = "f09f8dbc0000";
// const BASIC_MAGIC = MAGIC_STRING + "e000";
//
//
// describe("bottleWriter", () => {
//   it("writes a bottle header", future(() => {
//     const h = new Header();
//     h.addNumber(0, 150);
//     const b = writeBottle(10, h);
//     b.end();
//     return pipeToBuffer(b).then(data => {
//       data.toString("hex").should.eql(`${MAGIC_STRING}a003800196ff`);
//     });
//   }));
//
//   it("writes data", future(() => {
//     const data = sourceStream(new Buffer("ff00ff00", "hex"));
//     const b = writeBottle(10, new Header());
//     b.write(data);
//     b.end();
//     return pipeToBuffer(b).then(data => {
//       data.toString("hex").should.eql(`${MAGIC_STRING}a00004ff00ff0000ff`);
//     });
//   }));
//
//   it("writes nested bottle data", future(() => {
//     const b = new writeBottle(10, new Header());
//     const b2 = new writeBottle(14, new Header());
//     b.write(b2.pipe(bufferStream()));
//     b.end();
//     b2.end();
//     return pipeToBuffer(b).then(data => {
//       data.toString("hex").should.eql(`${MAGIC_STRING}a00009${MAGIC_STRING}e000ff00ff`);
//     });
//   }));
//
//   it("streams data", future(() => {
//     // just to verify that the data is written as it comes in, and the event isn't triggered until completion.
//     const data = new Buffer("c44c", "hex");
//     const slowStream = new stream.Readable();
//     slowStream._read = () => null;
//     slowStream.push(data);
//     const b = new writeBottle(14, new Header());
//     Promise.delay(100).then(() => {
//       slowStream.push(data);
//       Promise.delay(100).then(() => {
//         slowStream.push(null);
//       });
//     });
//     b.write(slowStream.pipe(bufferStream()));
//     b.end();
//     return pipeToBuffer(b).then(data => {
//       data.toString("hex").should.eql(`${MAGIC_STRING}e00004c44cc44c00ff`);
//     });
//   }));
//
//   it("writes several datas", future(() => {
//     const data1 = sourceStream(new Buffer("f0f0f0", "hex"));
//     const data2 = sourceStream(new Buffer("e0e0e0", "hex"));
//     const data3 = sourceStream(new Buffer("cccccc", "hex"));
//     const b = writeBottle(14, new Header());
//     b.write(data1);
//     b.write(data2);
//     b.write(data3);
//     b.end();
//     return pipeToBuffer(b).then(data => {
//       data.toString("hex").should.eql(`${MAGIC_STRING}e00003f0f0f00003e0e0e00003cccccc00ff`);
//     });
//   }));
// });
//
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
