use bytes::{Bytes};
use futures::{Async, Future, Poll, Stream, stream};
use futures::stream::{Fuse};
use std::collections::VecDeque;
use std::io;

/*
 * wrapper for Stream where you may call `read` to get specific-sized buffers
 * out.
 *
 * FIXME: i bet buffer_stream could be implemented using this.
 */
pub fn stream_reader<S>(s: S) -> StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  StreamReader::new(s)
}

pub struct StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  stream: Fuse<S>,
  saved: VecDeque<Bytes>,
  total_saved: usize,
  error: Option<io::Error>
}

impl<S> StreamReader<S> where S: Stream<Item = Bytes, Error = io::Error> {
  pub fn new(s: S) -> StreamReader<S> {
    StreamReader {
      stream: s.fuse(),
      saved: VecDeque::new(),
      total_saved: 0,
      error: None
    }
  }

  /*
   * "read" up to `count` bytes by shuffling `Bytes` objects (without
   * copying buffers).
   *   - if `at_most` is true, a `Bytes` may be split to return exactly
   *     `count` bytes.
   *   - if there aren't `count` bytes buffered, you'll get less than you
   *     asked for. to prevent this, check `total_saved` before calling.
   */
  fn drain(&mut self, count: usize, at_most: bool) -> Vec<Bytes> {
    let mut rv: Vec<Bytes> = Vec::new();
    let mut so_far = 0;

    while self.saved.len() > 0 && so_far < count {
      let chunk = self.saved.pop_front().unwrap();
      if (so_far + chunk.len() <= count) || !at_most {
        so_far += chunk.len();
        self.total_saved -= chunk.len();
        rv.push(chunk);
      } else {
        let n = count - so_far;
        so_far += n;
        self.total_saved -= n;
        rv.push(chunk.slice(0, n));
        self.saved.push_front(chunk.slice_from(n));
      }
    }

    rv
  }

  pub fn read_exact<'a>(&'a mut self, count: usize) -> impl 'a + Future<Item = Vec<Bytes>, Error = io::Error> {
    StreamReaderFuture::<'a> { reader: self, count: count, at_most: true, at_least: true }
  }

  pub fn read<'a>(&'a mut self, count: usize) -> impl 'a + Future<Item = Vec<Bytes>, Error = io::Error> {
    StreamReaderFuture::<'a> { reader: self, count: count, at_most: true, at_least: false }
  }

  // convert any un-read data back into a stream.
  pub fn into_stream(self) -> impl Stream<Item = Bytes, Error = io::Error> {
    stream::iter(self.saved.into_iter().map(|b| Ok(b))).chain(self.stream)
  }
}


// ----- StreamReaderFuture

pub struct StreamReaderFuture<'a, S> where S: 'a + Stream<Item = Bytes, Error = io::Error> {
  reader: &'a mut StreamReader<S>,
  count: usize,
  at_most: bool,    // do not return more than `count` bytes.
  at_least: bool    // do not return less than `count` bytes (error on EOF).
}

impl<'a, S> Future for StreamReaderFuture<'a, S> where S: Stream<Item = Bytes, Error = io::Error> {
  type Item = Vec<Bytes>;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    loop {
      if self.reader.total_saved >= self.count {
        return Ok(Async::Ready(self.reader.drain(self.count, self.at_most)))
      }

      if let Some(error) = self.reader.error.take() {
        // if there's no minimum len, drain the remainder before reporting the error.
        if self.reader.saved.len() > 0 && !self.at_least {
          return Ok(Async::Ready(self.reader.drain(self.count, self.at_most)));
        } else {
          return Err(error);
        }
      }

      match self.reader.stream.poll() {
        Ok(Async::NotReady) => {
          return Ok(Async::NotReady);
        }

        // end of stream
        Ok(Async::Ready(None)) => {
          if (self.at_least && (self.reader.total_saved < self.count)) || self.reader.total_saved == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
          } else {
            return Ok(Async::Ready(self.reader.drain(self.count, self.at_most)));
          }
        }

        Ok(Async::Ready(Some(buffer))) => {
          self.reader.total_saved += buffer.len();
          self.reader.saved.push_back(buffer);
          // fall through to check if we have enough buffered to exit.
        }

        // in rust streams, errors float downsteam as if they were items
        // it's really fucking weird. mimic the streams library by allowing
        // any buffered data to be processed before puking the error.
        Err(error) => {
          self.reader.error = Some(error);
          // fall through to drain all the pre-error buffers.
        }
      }
    }
  }
}
