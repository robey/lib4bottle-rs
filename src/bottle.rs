use futures::{Future, future, Stream, stream};
use std::io;
use std::iter::Iterator;
use bytes::Bytes;

use buffered_stream::{BufferedStream};
use header::{BottleType, Header};
use stream_helpers::{stream_of, stream_of_vec};
// use stream_helpers::{flatten_bytes, make_framed_stream_1, make_framed_stream_3, make_stream_1};
use stream_reader::{ByteFrame, StreamReader};
use table::{Table};
use zint;


const MIN_BUFFER: usize = 1024;

lazy_static! {
  static ref END_OF_STREAM_BYTES: Bytes = Bytes::from(zint::encode_length(zint::END_OF_STREAM));
  static ref END_OF_ALL_STREAMS_BYTES: Bytes = Bytes::from(zint::encode_length(zint::END_OF_ALL_STREAMS));
}

/// Bottle of some known type, metadata table, and a "stream of streams".
pub struct Bottle<S, SS>
  where
    S: Stream<Item = Bytes, Error = io::Error>,
    SS: Stream<Item = S, Error = io::Error>
{
  header: Header,
  streams: SS
}

impl<S, SS> Bottle<S, SS>
  where
    S: Stream<Item = Bytes, Error = io::Error>,
    SS: Stream<Item = S, Error = io::Error>
{
  pub fn new(bottle_type: BottleType, table: Table, streams: SS) -> Bottle<S, SS> {
    Bottle { header: Header::new(bottle_type, table), streams }
  }

  /// Consume the streams by encoding everything into one big happy byte
  /// stream.
  pub fn encode(self) -> impl Stream<Item = Bytes, Error = io::Error> {
    self.header.encode().chain(self.streams.map(|s| {
      frame_stream(s)
    }).flatten()).chain(stream_of(END_OF_ALL_STREAMS_BYTES.clone()))
  }
}



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



/// Convert a byte stream into a stream with each chunk prefixed by a length
/// marker, suitable for embedding in a bottle. Buffering converts clusters
/// of small blocks into a single "frame" that we can serialize, without
/// copying the buffers around.
pub fn frame_stream<S>(s: S) -> impl Stream<Item = Bytes, Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  // prevent tiny packets by requiring it to buffer at least 1KB
  BufferedStream::new(s, MIN_BUFFER, false).map(|frame| {
    let prefix: Bytes = Bytes::from(zint::encode_length(frame.length as u32));
    // transform frame into Stream<Bytes>:
    stream_of(prefix).chain(stream_of_vec(frame.vec))
  }).flatten().chain(stream_of(END_OF_STREAM_BYTES.clone()))
}

// export function unframingStream() {
//   const readLength = t => {
//     return t.get(1).then(byte => {
//       if (byte == null || byte.length < 1) return null;
//       const needed = lengthLength(byte[0]) - 1;
//       if (needed == 0) return decodeLength(byte);
//
//       return t.get(needed).then(rest => {
//         if (rest == null || rest.length < needed) return null;
//         return decodeLength(Buffer.concat([ byte, rest ]));
//       });
//     });
//   };
//
//   const transform = new PullTransform({
//     name: "unframingStream",
//     transform: t => {
//       return readLength(t).then(length => {
//         if (length == null || length <= 0) {
//           t.push(null);
//           return;
//         }
//         return t.get(length);
//       });
//     }
//   });
//
//   return promisify(transform, { name: "unframingStream" });
// }
//



// ----- errors






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
