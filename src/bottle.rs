use bytes::Bytes;
use futures::{Future, future, IntoFuture, Stream, stream};
use std::io;

use header::{BottleType, Header};
use stream_toolkit::{
  BufferedByteStream, ByteFrame, generate_stream, OptionToFuture, ReadableByteStream, stream_of, stream_of_vec
};
use table::Table;
use zint;

const MIN_BUFFER: usize = 1024;

/// Bottle of some known type, metadata table, and a "stream of streams".
pub struct Bottle<S, SS>
  where
    S: Stream<Item = Bytes, Error = io::Error>,
    SS: Stream<Item = S, Error = io::Error>
{
  pub header: Header,
  pub streams: SS
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
    let header_stream = self.header.encode();
    let streams_stream = self.streams.map(|s| write_framed_stream(s)).flatten();
    let tail_stream = stream_of(zint::END_OF_BOTTLE_BYTES.clone());

    header_stream.chain(streams_stream).chain(tail_stream)
  }
}


// bottle::Bottle<
//   impl futures::Stream,
//   futures::stream::Map<
//     stream_toolkit::stream_generator::StreamGenerator<
//       stream_toolkit::readable_byte_stream::ReadableByteStream<S>,
//       [closure@src/bottle.rs:58:48: 60:6],
//       impl futures::Future,
//       impl futures::Future
//     >,
//     [closure@src/bottle.rs:61:56: 61:75]
//   >
// >

// bottle::Bottle<
//   stream_toolkit::readable_byte_stream::ReadableByteStream<impl futures::Stream>,
//   stream_toolkit::stream_generator::StreamGenerator<
//     stream_toolkit::readable_byte_stream::ReadableByteStream<S>,
//     [closure@src/bottle.rs:71:48: 73:6],
//     impl futures::Future,
//     impl futures::Future
//   >
// >


pub fn read_bottle<S>(s: ReadableByteStream<S>)
  -> impl Future<
    Item = (
      Bottle<
        impl Stream<Item = Bytes, Error = io::Error>,
        impl Stream<Item = impl Stream<Item = Bytes, Error = io::Error>, Error = io::Error>
      >,
      impl Future<Item = ReadableByteStream<S>>
    ),
    Error = io::Error
  >
  where
    S: Stream<Item = Bytes, Error = io::Error>
{
  Header::decode(s).map(|(header, s)| {
    let (streams, future) = generate_stream(s, |s| {
      read_framed_stream(s)
    });

    let bottle = Bottle { header, streams: streams.map(|s| s.into_stream()) };
    ( bottle, future )
  })
}

fn read_streams<S>(s: ReadableByteStream<S>)
  -> (
    impl Stream<Item = impl Stream<Item = Bytes, Error = io::Error>, Error = io::Error>,
    impl Future<Item = ReadableByteStream<S>>
  )
  where S: Stream<Item = Bytes, Error = io::Error>
{
  let (stream, future) = generate_stream(s, |s| read_framed_stream(s));
  ( stream.map(|s| s.into_stream()), future )
}

fn read_streams2<S, X>(s: ReadableByteStream<S>)
  -> (
    impl Stream<Item = X, Error = io::Error>,
    impl Future<Item = ReadableByteStream<S>>
  )
  where
    S: Stream<Item = Bytes, Error = io::Error>,
    X: Stream<Item = Bytes, Error = io::Error>,
{
  let (stream, future) = generate_stream(s, |s| read_framed_stream(s));
  ( stream.map(|s| s.into_stream()), future )
}

// pub fn read_framed_stream<S>(s: ReadableByteStream<S>)
//   -> impl Future<Item = (
//     Option<ReadableByteStream<impl Stream<Item = Bytes, Error = io::Error>>>,
//     impl Future<Item = ReadableByteStream<S>, Error = io::Error>
//   ), Error = io::Error>
//   where S: Stream<Item = Bytes, Error = io::Error>
//


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
pub fn write_framed_stream<S>(s: S) -> impl Stream<Item = Bytes, Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  // prevent tiny packets by requiring it to buffer at least 1KB
  BufferedByteStream::new(s, MIN_BUFFER, false).map(|frame| {
    let prefix: Bytes = Bytes::from(zint::encode_length(frame.length));
    // transform frame into Stream<Bytes>:
    stream_of(prefix).chain(stream_of_vec(frame.vec))
  }).flatten().chain(stream_of(zint::END_OF_STREAM_BYTES.clone()))
}

/// Read a framed stream and transform it back into a normal byte stream.
/// Returns the nested stream, and a future that will resolve to the
/// remainder of the original stream, once the inner stream is drained.
/// If we hit the end-of-all-streams marker, no nested stream is returned.
pub fn read_framed_stream<S>(s: ReadableByteStream<S>)
  -> impl Future<Item = (
    Option<ReadableByteStream<impl Stream<Item = Bytes, Error = io::Error>>>,
    impl Future<Item = ReadableByteStream<S>, Error = io::Error>
  ), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  is_end_of_bottle(s).map(|(is_end, s)| {
    let (stream, future) = generate_stream(s, |s| {
      read_frame(s).map(|(length, frame, s)| {
        if length == zint::FrameLength::EndOfStream || length == zint::FrameLength::EndOfBottle {
          (None, future::ok(s))
        } else {
          (Some(frame), future::ok(s))
        }
      })
    });

    let (possibly_drain, stream): (Option<stream::Collect<_>>, Option<ReadableByteStream<_>>) = if is_end {
      ( Some(stream.collect()), None )
    } else {
      ( None, Some(ReadableByteStream::from(ByteFrame::flatten_stream(stream))) )
    };
    ( stream, possibly_drain.to_future().and_then(|_| future) )
  })
}

fn read_frame<S>(s: ReadableByteStream<S>)
  -> impl Future<Item = (zint::FrameLength, ByteFrame, ReadableByteStream<S>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  read_frame_length(s).and_then(|(length, s)| {
    let count: usize = match length {
      zint::FrameLength::Length(n) => n,
      _ => 0
    };
    s.read_exact(count).map(|(frame, s)| {
      ( length, frame, s )
    })
  })
}

fn read_frame_length<S>(s: ReadableByteStream<S>)
  -> impl Future<Item = (zint::FrameLength, ReadableByteStream<S>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  s.read_exact(1).and_then(|(frame, s)| {
    let byte: u8 = frame.vec[0][0];
    let ( count, accumulator ) = zint::decode_first_length_byte(byte);
    s.read_exact(count).map(|(frame, s)| {
      ( zint::decode_length(accumulator, frame.pack().as_ref()), s )
    })
  })
}

fn is_end_of_bottle<S>(s: ReadableByteStream<S>)
  -> impl Future<Item = (bool, ReadableByteStream<S>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  s.read_exact(1).map(|(frame, mut s)| {
    let byte: u8 = frame.vec[0][0];
    let ( _, accumulator ) = zint::decode_first_length_byte(byte);
    let is_end = accumulator == zint::FrameLength::EndOfBottle;
    s.unread(frame);
    ( is_end, s )
  })
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
//
