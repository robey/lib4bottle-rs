use bytes::{Bytes};
use futures::{Async, Future, Poll, Stream, stream};
use futures::stream::{Fuse};
use std::collections::VecDeque;
use std::io;

/*
 * Read exactly `count` bytes from a stream of `Bytes` objects, returning a
 * `Vec<Bytes>` containing the cumulative buffers totalling exactly the
 * desired bytes, and a new `Stream` representing everything afterwards.
 * If not enough bytes are available on the stream before EOF, an EOF error
 * is returned.
 */
pub fn stream_read_exact<S>(s: S, count: usize)
  -> impl Future<Item = (Vec<Bytes>, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  read_from_stream(s, count, true, true)
}

/*
 * Read no more than `count` bytes from a stream of `Bytes` objects, returning
 * a `Vec<Bytes>` containing the cumulative buffers totalling up to (but no
 * more than) the desired bytes, and a new `Stream` representing everything
 * afterwards. If fewer bytes are returned, the stream hit EOF.
 */
pub fn stream_read<S>(s: S, count: usize)
  -> impl Future<Item = (Vec<Bytes>, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  read_from_stream(s, count, true, false)
}

/*
 * Read at least `count` bytes from a stream of `Bytes` objects, if possible,
 * returning a `Vec<Bytes>` containing the cumulative buffers, and a new
 * `Stream` representing everything afterwards. If fewer bytes are returned,
 * the stream hit EOF. More bytes may be returned if it would avoid splitting
 * a `Bytes` object.
 */
pub fn stream_read_buffered<S>(s: S, block_size: usize)
  -> impl Future<Item = (Vec<Bytes>, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  read_from_stream(s, block_size, false, false)
}

// (stream, count, at_most, at_least) -> (vec, stream)
// any remainder has been prefixed back into the returned stream.
fn read_from_stream<S>(s: S, count: usize, at_most: bool, at_least: bool)
  -> impl Future<Item = (Vec<Bytes>, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  read_from_stream_raw(s, count, at_most, at_least).map(|(buffer, remainder, stream)| {
    let merged = stream::iter(remainder.into_iter().map(|b| Ok(b))).chain(stream);
    ( buffer, merged )
  })
}

// lowest-level call. perform a read, then return the read, any remainder, and the original stream.
fn read_from_stream_raw<S>(s: S, count: usize, at_most: bool, at_least: bool)
  -> impl Future<Item = (Vec<Bytes>, Option<Bytes>, S), Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  StreamReader {
    stream: Some(s.fuse()),
    count: count,
    at_most: at_most,
    at_least: at_least,
    saved: VecDeque::new(),
    total_saved: 0,
    error: None
  }
}

#[must_use = "futures do nothing unless polled"]
struct StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  stream: Option<Fuse<S>>,
  count: usize,
  at_most: bool,
  at_least: bool,

  // internal state:
  saved: VecDeque<Bytes>,
  total_saved: usize,
  error: Option<io::Error>
}

impl<S> StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  /*
   * "read" up to `count` bytes by shuffling `Bytes` objects, returning a
   * vector to avoid buffer-copying. you can flatten the result at your
   * leisure if that's what you want.
   *   - if `at_most` is true, a `Bytes` may be split to return exactly
   *     `count` bytes.
   *   - if there aren't `count` bytes buffered, you'll get less than you
   *     asked for. to prevent this, check `total_saved` before calling.
   */
  fn drain(&mut self) -> Vec<Bytes> {
    let mut rv: Vec<Bytes> = Vec::new();
    let mut so_far = 0;

    while self.saved.len() > 0 && so_far < self.count {
      let chunk = self.saved.pop_front().unwrap();
      if (so_far + chunk.len() <= self.count) || !self.at_most {
        so_far += chunk.len();
        self.total_saved -= chunk.len();
        rv.push(chunk);
      } else {
        let n = self.count - so_far;
        so_far += n;
        self.total_saved -= n;
        rv.push(chunk.slice(0, n));
        self.saved.push_front(chunk.slice_from(n));
      }
    }

    rv
  }

  /*
   * assuming we have collected as many bytes as we need, or as many bytes
   * as we CAN, drain off enough `Bytes` objects to fill the original request
   * (or try), return any unused buffer (there should be at most one), and
   * return the original stream.
   */
  fn complete(&mut self) -> (Vec<Bytes>, Option<Bytes>, S) {
    let buffer = self.drain();
    assert!(self.saved.len() <= 1);
    let remainder = self.saved.pop_front();
    let stream = self.stream.take().unwrap().into_inner();
    (buffer, remainder, stream)
  }
}

impl<S> Future for StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  type Item = (Vec<Bytes>, Option<Bytes>, S);
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      if self.total_saved >= self.count {
        return Ok(Async::Ready(self.complete()))
      }

      if let Some(error) = self.error.take() {
        // if there's no minimum len, drain the remainder before reporting the error.
        if self.saved.len() > 0 && !self.at_least {
          return Ok(Async::Ready(self.complete()));
        } else {
          return Err(error);
        }
      }

      match self.stream.as_mut().expect("polling stream twice").poll() {
        Ok(Async::NotReady) => {
          return Ok(Async::NotReady);
        }

        // end of stream
        Ok(Async::Ready(None)) => {
          if self.at_least && (self.total_saved < self.count) {
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
        // it's really fucking weird. mimic the streams library by allowing
        // any buffered data to be processed before puking the error.
        Err(error) => {
          self.error = Some(error);
          // fall through to drain all the pre-error buffers.
        }
      }
    }
  }
}
