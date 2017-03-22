use bytes::{Bytes};
use futures::{Async, Future, Poll, Stream, stream};
use futures::stream::{Fuse};
use std::collections::VecDeque;
use std::io;

#[derive(Clone, Copy, PartialEq)]
pub enum StreamReaderMode {
  /// Return exactly the number of bytes requested, no more, no less. If
  /// there aren't enough bytes before the end of the stream, return an error.
  Exact,

  /// Return no more than the number of bytes requested. Fewer is okay, if
  /// the stream ends prematurely.
  AtMost,

  /// Return at least as many bytes as requested, if possible. Fewer is okay,
  /// if the stream ends prematurely. More is okay, if it would otherwise
  /// break up a buffer.
  Lazy
}

#[must_use = "futures do nothing unless polled"]
pub struct StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  stream: Option<Fuse<S>>,
  count: usize,
  mode: StreamReaderMode,

  // internal state:
  saved: VecDeque<Bytes>,
  total_saved: usize
}

impl<S> StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  /// Read `count` bytes from a stream, optionally prefixed with a buffer
  /// left over from a previous read. This operation consumes the stream, but
  /// returns ownership once the future is completed.
  ///
  /// Returns a `Future` containing a frame (`Vec<Bytes>`), an optional
  /// remainder, and the original stream. The remainder will be present if
  /// it was necessary to split a `Bytes` in order to return an exact buffer
  /// size. This remainder can be passed into future calls as the prefix, or
  /// merged back into the original stream by calling `merge` on the result.
  pub fn read(s: S, count: usize, mode: StreamReaderMode, prefix: Option<Bytes>)
    -> StreamReader<S>
  {
    let mut saved = VecDeque::new();
    let total_saved = prefix.clone().map_or(0, |b| b.len());
    saved.extend(prefix.into_iter());
    StreamReader {
      stream: Some(s.fuse()),
      count: count,
      mode: mode,
      saved: saved,
      total_saved: total_saved
    }
  }

  /// Read exactly `count` bytes from a stream, returning a `ByteFrame`
  /// containing the cumulative buffers totalling exactly the desired bytes,
  /// and a new `Stream` representing everything afterwards.
  ///
  /// If not enough bytes are available on the stream before EOF, an EOF error
  /// is returned.
  pub fn read_exact(s: S, count: usize)
    -> impl Future<Item = (ByteFrame, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
  {
    StreamReader::read(s, count, StreamReaderMode::Exact, None).map(|result| {
      result.into_stream()
    })
  }

  /// Read at most `count` bytes from a stream, returning a `ByteFrame`
  /// containing the cumulative buffers, and a new `Stream` representing
  /// everything afterwards.
  ///
  /// If not enough bytes are available on the stream before EOF, the frame
  /// may contain fewer bytes than requested.
  pub fn read_at_most(s: S, count: usize)
    -> impl Future<Item = (ByteFrame, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
  {
    StreamReader::read(s, count, StreamReaderMode::AtMost, None).map(|result| {
      result.into_stream()
    })
  }

  /// Drain up to `count` bytes from the saved deque, returning a new vector
  /// to avoid copying buffers.
  ///
  /// - If `mode` is `Exact` or `AtMost`, a `Bytes` may be split to return
  ///   exactly `count` bytes.
  /// - If there aren't `count` bytes buffered, you'll get less than you
  ///   asked for. To prevent this, check `total_saved` before calling.
  fn drain(&mut self) -> ByteFrame {
    let mut vec: Vec<Bytes> = Vec::new();
    let mut length = 0;

    while self.saved.len() > 0 && length < self.count {
      let chunk = self.saved.pop_front().unwrap();
      if (length + chunk.len() <= self.count) || self.mode == StreamReaderMode::Lazy {
        length += chunk.len();
        self.total_saved -= chunk.len();
        vec.push(chunk);
      } else {
        let n = self.count - length;
        length += n;
        self.total_saved -= n;
        vec.push(chunk.slice(0, n));
        self.saved.push_front(chunk.slice_from(n));
      }
    }

    ByteFrame { vec: vec, length: length }
  }

  /*
   * assuming we have collected as many bytes as we need, or as many bytes
   * as we CAN, drain off enough `Bytes` objects to fill the original request
   * (or try), return any unused buffer (there should be at most one), and
   * return the original stream.
   */
  fn complete(&mut self) -> StreamReaderResult<S> {
    let frame = self.drain();
    assert!(self.saved.len() <= 1);
    let remainder = self.saved.pop_front();
    let stream = self.stream.take().unwrap().into_inner();
    StreamReaderResult { frame: frame, remainder: remainder, stream: stream }
  }
}

impl<S> Future for StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  type Item = StreamReaderResult<S>;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      if self.total_saved >= self.count {
        return Ok(Async::Ready(self.complete()))
      }

      match self.stream.as_mut().expect("polling stream twice").poll() {
        Ok(Async::NotReady) => {
          return Ok(Async::NotReady);
        }

        // end of stream
        Ok(Async::Ready(None)) => {
          if self.mode == StreamReaderMode::Exact && (self.total_saved < self.count) {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
          } else {
            return Ok(Async::Ready(self.complete()));
          }
        }

        Ok(Async::Ready(Some(buffer))) => {
          self.total_saved += buffer.len();
          self.saved.push_back(buffer);
          // fall through to check if we have enough buffered to exit.
        }

        // in rust streams, errors float downsteam as if they were items.
        // i don't believe in that, so treat any error as if the stream has
        // crashed.
        Err(error) => {
          return Err(error);
        }
      }
    }
  }
}


// ----- StreamReaderResult

pub struct StreamReaderResult<S> where S: Stream<Item = Bytes, Error = io::Error> {
  pub frame: ByteFrame,
  pub remainder: Option<Bytes>,
  pub stream: S
}

impl<S> StreamReaderResult<S> where S: Stream<Item = Bytes, Error = io::Error> {
  /// Merge any remainder buffer back into the stream as if it had been
  /// "un-read". This consumes the result, returning the frame and the new
  /// combined stream.
  pub fn into_stream(self) -> (ByteFrame, impl Stream<Item = Bytes, Error = io::Error>) {
    ( self.frame, stream::iter(self.remainder.into_iter().map(|b| Ok(b))).chain(self.stream) )
  }
}


// ----- ByteFrame

pub struct ByteFrame {
  pub vec: Vec<Bytes>,
  pub length: usize
}

impl ByteFrame {
  pub fn new(vec: Vec<Bytes>, length: usize) -> ByteFrame {
    ByteFrame { vec: vec, length: length }
  }
}
