use bytes::Bytes;
use futures::{Future, future, Stream, stream};
use std::io;

use header::{BottleType, Header};
use stream_toolkit::{
  BufferedByteStream, ByteFrame, generate_stream, OptionToFuture, ReadableByteStream, stream_of, stream_of_vec
};
use table::Table;
use zint;

const MIN_BUFFER: usize = 1024;

/// Bottle of some known type, metadata table, and a "stream of streams".
pub struct Bottle<S>
  where
    S: Stream<Error = io::Error>,
    S::Item: Stream<Item = Bytes, Error = io::Error>,
{
  pub header: Header,
  pub streams: S
}

impl<S> Bottle<S>
  where
    S: Stream<Error = io::Error>,
    S::Item: Stream<Item = Bytes, Error = io::Error>,
{
  pub fn new(bottle_type: BottleType, table: Table, streams: S) -> Bottle<S> {
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

/// Read a bottle out of a byte stream, returning a future of the bottle, and
/// any stream remaining after the end of the bottle.
pub fn read_bottle<S>(s: ReadableByteStream<S>)
  -> impl Future<Item = (
    Bottle<impl Stream<Item = impl Stream<Item = Bytes, Error = io::Error>, Error = io::Error>>,
    impl Future<Item = ReadableByteStream<S>>
  ), Error = io::Error>
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
/// If we hit the end-of-all-streams marker (signifying the end of the
/// bottle), `None` is returned. Otherwise `Some(stream)` is returned.
/// In either case, the original stream is returned as a future that will
/// resolve once the inner stream has been drained.
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
