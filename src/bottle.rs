
use futures::{Stream, stream};
use std::io;
use std::iter::Iterator;
use bytes::Bytes;

use bottle_header::{Header};
use stream_helpers::{make_stream_1, make_stream_3};
use zint;

static MAGIC: [u8; 4] = [ 0xf0, 0x9f, 0x8d, 0xbc ];
const VERSION: u8 = 0;

const MAX_HEADER_SIZE: usize = 4095;

pub enum BottleType {
  File = 0,
  Hashed = 1,
  Encrypted = 3,
  Compressed = 4,
  // for tests:
  Test = 10
}

// generate a bottle from a type, header, and a list of streams.
pub fn make_bottle<I, A>(btype: BottleType, header: &Header, streams: I)
  -> impl Stream<Item = Vec<Bytes>, Error = io::Error>
  where
    I: IntoIterator<Item = A>,
    A: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  // FIXME: static
  let end_of_all_streams = make_stream_1(Bytes::from(zint::encode_length(zint::END_OF_ALL_STREAMS)));
  let combined = stream::iter(streams.into_iter().map(|s| Ok::<A, io::Error>(s))).flatten();
  make_header_stream(btype, header).chain(combined).chain(end_of_all_streams)
}

// /*
//  * Stream transform that accepts child streams and emits them as a single
//  * bottle stream with a header.
//  */
// export function writeBottle(type, header, options = {}) {
//   const streamOptions = {
//     name: "bottleWriterGuts",
//     writableObjectMode: true,
//     readableObjectMode: true,
//     highWaterMark: STREAM_BUFFER_SIZE,
//     transform: inStream => {
//       // prevent tiny packets by requiring it to buffer at least 1KB
//       const buffered = bufferStream(MIN_BUFFER);
//       const framedStream = framingStream();
//       transform.__log("writing stream " + (inStream.__name || "?") + " into " + framedStream.__name);
//       inStream.pipe(buffered);
//       buffered.pipe(framedStream);
//       return framedStream;
//     },
//     flush: () => {
//       transform.__log("flush: end of bottle");
//       return sourceStream(new Buffer([ BOTTLE_END ]));
//     }
//   };
//   for (const k in options) streamOptions[k] = options[k];
//
//   const transform = new Transform(streamOptions);
//   transform.push(sourceStream(writeHeader(type, header)));
//   const outStream = compoundStream();
//   return weld(transform, outStream, {
//     name: `BottleWriter(${bottleTypeName(type)}, ${options.tag || ""})`,
//     writableObjectMode: true
//   });
// }

// // convert a byte stream into a stream with each chunk prefixed by a length
// // marker, suitable for embedding in a bottle.
// pub fn framed_stream<S>(s: S) -> impl Stream<Item = Bytes, Error = io::Error>
//   where S: Stream<Item = Bytes, Error = io::Error>
// {
//   let end_of_stream = make_stream_1(Bytes::from(zint::encode_length(zint::END_OF_STREAM)));
//   s.map(|buffer| {
//     make_stream_2(Bytes::from(zint::encode_length(buffer.len() as u32)), buffer)
//   }).flatten().chain(end_of_stream)
// }

// convert a byte stream into a stream with each chunk prefixed by a length
// marker, suitable for embedding in a bottle.
pub fn framed_vec_stream<S>(s: S) -> impl Stream<Item = Vec<Bytes>, Error = io::Error>
  where S: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  // FIXME: static
  let end_of_stream = make_stream_1(Bytes::from(zint::encode_length(zint::END_OF_STREAM)));
  s.map(|buffers| {
    let mut new_buffers = Vec::with_capacity(buffers.len() + 1);
    let total_length: usize = buffers.iter().fold(0, |sum, buf| sum + buf.len());
    new_buffers.push(Bytes::from(zint::encode_length(total_length as u32)));
    new_buffers.extend(buffers);
    new_buffers
  }).chain(end_of_stream)
}

// generate a stream that's just a bottle header (magic + header data).
pub fn make_header_stream(btype: BottleType, header: &Header) -> impl Stream<Item = Vec<Bytes>, Error = io::Error> {
  let headerBytes = header.encode();
  assert!(headerBytes.len() <= MAX_HEADER_SIZE);
  let version: [u8; 4] = [
    VERSION,
    0,
    ((btype as u8) << 4) | ((headerBytes.len() >> 8) & 0xf) as u8,
    (headerBytes.len() & 0xff) as u8
  ];
  make_stream_3(Bytes::from_static(&MAGIC), Bytes::from(&version[..]), Bytes::from(headerBytes))
}




/*
 * Stream transform that prefixes each buffer with a length header so it can
 * be streamed. If you want to create large frames, pipe through a
 * bufferingStream first.
 */


// impl futures::Sink for BottleSink {
//   type SinkItem = futures::stream::BoxStream<Vec<u8>, io::Error>;
//   type SinkError = io::Error;
//
//   pub start_send(&mut self, item: Self::SinkItem) {
//   }
// }



// import { bufferStream, compoundStream, PullTransform, sourceStream, Transform, weld } from "stream-toolkit";
// import { packHeader, unpackHeader } from "./bottle_header";
// import { framingStream, unframingStream } from "./framed_stream";
//
//
//
// const MIN_BUFFER = 1024;
// const STREAM_BUFFER_SIZE = 256 * 1024;
//
// export function bottleTypeName(n) {
//   switch (n) {
//     case TYPE_FILE: return "file";
//     case TYPE_HASHED: return "hashed";
//     case TYPE_ENCRYPTED: return "encrypted";
//     case TYPE_COMPRESSED: return "compressed";
//     default: return n.toString();
//   }
// }
//
// const BOTTLE_END = 0xff;
//
//
// /*
//  * Stream transform that accepts a byte stream and emits a header, then one
//  * or more child streams.
//  */
// export function readBottle(options = {}) {
//   const streamOptions = {
//     readableObjectMode: true,
//     highWaterMark: STREAM_BUFFER_SIZE,
//     transform: t => {
//       return readHeader(t).then(header => {
//         t.push(header);
//         return next(t);
//       });
//     }
//   };
//   for (const k in options) streamOptions[k] = options[k];
//   return new PullTransform(streamOptions);
//
//   function next(t) {
//     return t.get(1).then(byte => {
//       if (!byte || byte[0] == BOTTLE_END) {
//         t.push(null);
//         return;
//       }
//       // put it back. it's part of a data stream!
//       t.unget(byte);
//
//       // unframe and emit.
//       const unframing = unframingStream();
//       t.subpipe(unframing);
//       t.push(unframing);
//       return unframing.endPromise().then(() => next(t));
//     });
//   }
// }
//
// function readHeader(transform) {
//   transform.__log("readBottleHeader");
//   return transform.get(8).then(buffer => {
//     if (!buffer || buffer.length < 8) throw new Error("End of stream");
//     for (let i = 0; i < 4; i++) {
//       if (buffer[i] != MAGIC[i]) throw new Error("Incorrect magic (not a 4bottle archive)");
//     }
//     if (buffer[4] != VERSION) throw new Error(`Incompatible version: ${buffer[4].toString(16)}`);
//     if (buffer[5] != 0) throw new Error(`Incompatible flags: ${buffer[5].toString(16)}`);
//     const type = (buffer[6] >> 4) & 0xf;
//     const headerLength = ((buffer[6] & 0xf) * 256) + (buffer[7] & 0xff);
//     return transform.get(headerLength).then(headerBuffer => {
//       const rv = { type, header: unpackHeader(headerBuffer || new Buffer(0)) };
//       if (transform.__debug) transform.__log("readBottleHeader -> " + type + ", " + rv.header.toString());
//       return rv;
//     });
//   });
// }
