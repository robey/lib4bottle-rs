use bytes::Bytes;
use std::io;
use futures::{Async, Future, Poll, Stream};

use stream_reader::{ByteFrame, StreamReader, StreamReaderMode, StreamReaderResult};

/// `Stream<Bytes>` that buffers data until it reaches a desired block size,
/// then emits a single `ByteFrame` (a vector of `Bytes`). If `exact` is set,
/// each block will be `block_size` bytes at most, even if it has to split up
/// a `Bytes`. (If we hit the end of the stream, the final block may be
/// smaller.)
#[must_use = "streams do nothing unless polled"]
pub struct BufferedStream<S> where S: Stream<Item = Bytes, Error = io::Error> {
  stream: Option<StreamReader<S>>,
  block_size: usize,
  mode: StreamReaderMode
}

impl<S> BufferedStream<S>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  pub fn new(s: S, block_size: usize, exact: bool) -> BufferedStream<S> {
    assert!(block_size > 0);
    let mode = if exact { StreamReaderMode::AtMost } else { StreamReaderMode::Lazy };
    BufferedStream {
      stream: Some(StreamReader::read(s, block_size, mode, None)),
      block_size: block_size,
      mode: mode
    }
  }

  pub fn pack(self) -> impl Stream<Item = Bytes, Error = io::Error> {
    self.map(|b| b.pack())
  }
}

impl<S> Stream for BufferedStream<S>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  type Item = ByteFrame;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    match self.stream.as_mut().expect("polling stream twice").poll() {
      Err(e) => Err(e),
      Ok(Async::NotReady) => Ok(Async::NotReady),
      Ok(Async::Ready(result)) => {
        let StreamReaderResult { frame, remainder, stream } = result;
        self.stream = Some(StreamReader::read(stream, self.block_size, self.mode, remainder));
        if frame.length == 0 { Ok(Async::Ready(None)) } else { Ok(Async::Ready(Some(frame))) }
      }
    }
  }
}
