use bytes::{Bytes};
use futures::{Async, Future, Poll, Stream, stream};
use futures::stream::{Fuse};
use std::collections::VecDeque;
use std::io;

/// Behaviors for `ReadableByteStream::read`
#[derive(Clone, Copy, PartialEq)]
pub enum ReadMode {
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

/// Wrap a `Stream<Bytes>` so that it has a few `read()` method variants,
/// each returning a future.
///
/// The stream is "consumed" while an outstanding future is executing, so
/// only one read may happen at once. Each future returns the frame that was
/// read, and a new `ReadableByteStream` representing the rest of the stream.
/// When you're done reading in this manner, the original stream can be
/// extracted, along with any remaining unused buffer.
///
/// Because `Bytes` objects may be split in the process of chopping them up
/// into perfectly-sized chunks, the object keeps pre-read data around to use
/// for subsequent requests. You may use `into_stream()` to create a `Stream`
/// object that combines the leftover buffers with the remaining stream.
pub struct ReadableByteStream<S> where S: Stream<Item = Bytes, Error = io::Error> {
  stream: Fuse<S>,
  saved: VecDeque<Bytes>,
  saved_count: usize
}

impl<S> ReadableByteStream<S> where S: Stream<Item = Bytes, Error = io::Error> {
  /// Read `count` bytes from a stream, returning a `Future<ByteFrame>` with
  /// a `Vec<Bytes>` of the cumulative buffers.
  pub fn read(self, count: usize, mode: ReadMode)
    -> ReadableByteStreamFuture<S>
  {
    ReadableByteStreamFuture {
      stream: Some(self.stream),
      count,
      mode,
      saved: self.saved,
      saved_count: self.saved_count
    }
  }

  /// Read exactly `count` bytes from a stream, returning a `ByteFrame`
  /// containing the cumulative buffers totalling exactly the desired bytes.
  /// If not enough bytes are available on the stream before EOF, an EOF error
  /// is returned.
  pub fn read_exact(self, count: usize)
    -> ReadableByteStreamFuture<S>
  {
    self.read(count, ReadMode::Exact)
  }

  /// Read at most `count` bytes from a stream, returning a `ByteFrame`
  /// containing the cumulative buffers.
  /// If not enough bytes are available on the stream before EOF, the frame
  /// may contain fewer bytes than requested.
  pub fn read_at_most(self, count: usize)
    -> ReadableByteStreamFuture<S>
  {
    self.read(count, ReadMode::AtMost)
  }

  /// Decompose back into an optional buffer (anything that has been pre-read)
  /// and the original stream.
  pub fn into_inner(self) -> (Option<Bytes>, S) {
    assert!(self.saved.len() <= 1);
    let mut saved = self.saved;
    ( saved.pop_front(), self.stream.into_inner() )
  }

  /// Merge any remainder buffer back into the stream as if it had been
  /// "un-read". This consumes `self`, returning the new combined stream.
  pub fn into_stream(self) -> impl Stream<Item = Bytes, Error = io::Error> {
    let stream = self.stream;
    stream::iter(self.saved.into_iter().map(|b| Ok(b))).chain(stream)
  }
}

impl<S> From<S> for ReadableByteStream<S> where S: Stream<Item = Bytes, Error = io::Error> {
  fn from(s: S) -> ReadableByteStream<S> {
    ReadableByteStream { stream: s.fuse(), saved: VecDeque::new(), saved_count: 0 }
  }
}


// ----- StreamReadFuture

#[must_use = "futures do nothing unless polled"]
pub struct ReadableByteStreamFuture<S> where S: Stream<Item = Bytes, Error = io::Error> {
  stream: Option<Fuse<S>>,
  count: usize,
  mode: ReadMode,

  // internal state:
  saved: VecDeque<Bytes>,
  saved_count: usize
}

impl<S> ReadableByteStreamFuture<S> where S: Stream<Item = Bytes, Error = io::Error> {
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
      if (length + chunk.len() <= self.count) || self.mode == ReadMode::Lazy {
        length += chunk.len();
        self.saved_count -= chunk.len();
        vec.push(chunk);
      } else {
        let n = self.count - length;
        length += n;
        self.saved_count -= n;
        vec.push(chunk.slice(0, n));
        self.saved.push_front(chunk.slice_from(n));
      }
    }

    ByteFrame { vec: vec, length: length }
  }

  /*
   * assuming we have collected as many bytes as we need, or as many bytes
   * as we CAN, drain off enough `Bytes` objects to fill the original request
   * (or try), and return a new `ReadableByteStream` representing the rest of
   * the stream.
   */
  fn complete(&mut self, stream: Fuse<S>) -> (ByteFrame, ReadableByteStream<S>) {
    let frame = self.drain();
    assert!(self.saved.len() <= 1);
    ( frame, ReadableByteStream { stream, saved: self.saved.clone(), saved_count: self.saved_count } )
  }
}

impl<S> Future for ReadableByteStreamFuture<S> where S: Stream<Item = Bytes, Error = io::Error> {
  type Item = (ByteFrame, ReadableByteStream<S>);
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      let mut stream = self.stream.take().expect("stream in use");
      if self.saved_count >= self.count {
        return Ok(Async::Ready(self.complete(stream)))
      }

      match stream.poll() {
        Ok(Async::NotReady) => {
          self.stream = Some(stream);
          return Ok(Async::NotReady);
        }

        // end of stream
        Ok(Async::Ready(None)) => {
          if self.mode == ReadMode::Exact && (self.saved_count < self.count) {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
          } else {
            return Ok(Async::Ready(self.complete(stream)));
          }
        }

        Ok(Async::Ready(Some(buffer))) => {
          self.stream = Some(stream);
          self.saved_count += buffer.len();
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


// ----- ByteFrame

/// A "frame" of bytes, consisting of a vector of `Bytes` objects and a
/// pre-calculated count of the total size.
pub struct ByteFrame {
  pub vec: Vec<Bytes>,
  pub length: usize
}

impl ByteFrame {
  pub fn new(vec: Vec<Bytes>, length: usize) -> ByteFrame {
    ByteFrame { vec: vec, length: length }
  }

  /// Convert a `Vec<Bytes>` into a `Bytes`, with copying. ☹️
  pub fn pack(&self) -> Bytes {
    if self.vec.len() == 1 {
      return self.vec[0].clone();
    }
    let len = self.vec.iter().fold(0, |sum, b| { sum + b.len() });
    let mut rv: Vec<u8> = Vec::with_capacity(len);
    for ref b in &self.vec { rv.extend(b.as_ref()) };
    Bytes::from(rv)
  }

  /// Convert a stream of `ByteFrame` into a stream of `Bytes` _without_ copying. 🎉
  pub fn flatten_stream<S, E>(s: S) -> impl Stream<Item = Bytes, Error = E>
    where S: Stream<Item = ByteFrame, Error = E>
  {
    s.map(|frame| stream::iter(frame.vec.into_iter().map(|b| Ok(b)))).flatten()
  }
}

impl From<Vec<Bytes>> for ByteFrame {
  fn from(v: Vec<Bytes>) -> ByteFrame {
    let length = v.iter().fold(0, |sum, b| sum + b.len());
    ByteFrame::new(v, length)
  }
}

impl From<Bytes> for ByteFrame {
  fn from(b: Bytes) -> ByteFrame {
    let length = b.len();
    ByteFrame::new(vec![ b ], length)
  }
}
