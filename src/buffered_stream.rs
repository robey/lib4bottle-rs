use bytes::Bytes;
use std::io;
use futures::{Async, Future, Poll, Stream};

use stream_reader::{ByteFrame, ReadableByteStream, ReadableByteStreamFuture, ReadMode};

/// `Stream<Bytes>` that buffers data until it reaches a desired block size,
/// then emits a single `ByteFrame` (a vector of `Bytes`). If `exact` is set,
/// each block will be `block_size` bytes at most, even if it has to split up
/// a `Bytes`. (If we hit the end of the stream, the final block may be
/// smaller.)
#[must_use = "streams do nothing unless polled"]
pub struct BufferedByteStream<S> where S: Stream<Item = Bytes, Error = io::Error> {
  future: Option<ReadableByteStreamFuture<S>>,
  block_size: usize,
  mode: ReadMode
}

impl<S> BufferedByteStream<S>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  pub fn new(s: S, block_size: usize, exact: bool) -> BufferedByteStream<S> {
    assert!(block_size > 0);
    let mode = if exact { ReadMode::AtMost } else { ReadMode::Lazy };
    BufferedByteStream {
      future: Some(ReadableByteStream::from(s).read(block_size, mode)),
      block_size: block_size,
      mode: mode
    }
  }

  pub fn pack(self) -> impl Stream<Item = Bytes, Error = io::Error> {
    self.map(|b| b.pack())
  }
}

impl<S> Stream for BufferedByteStream<S>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  type Item = ByteFrame;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    let mut future = self.future.take().expect("stream in use");
    match future.poll() {
      Err(e) => Err(e),
      Ok(Async::NotReady) => {
        self.future = Some(future);
        Ok(Async::NotReady)
      },
      Ok(Async::Ready((frame, stream))) => {
        self.future = Some(stream.read(self.block_size, self.mode));
        if frame.length == 0 { Ok(Async::Ready(None)) } else { Ok(Async::Ready(Some(frame))) }
      }
    }
  }
}
