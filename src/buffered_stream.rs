use bytes::Bytes;
use std::collections::VecDeque;
use std::io;
use futures::{Async, Poll, Stream};
use futures::stream::Fuse;

/*
 * Stream<Vec<Bytes>> that buffers data until it reaches a desired block size,
 * then emits a single block. If `exact` is set, each block will be exactly
 * `block_size`, even if it has to split up a `Bytes`. (In theory, this
 * doesn't copy buffers, just creates two slices that refer to the same
 * buffer.)
 */

#[must_use = "streams do nothing unless polled"]
pub struct BufferedStream<T> where T: Stream<Item = Vec<Bytes>, Error = io::Error> {
  items: VecDeque<Bytes>,
  total: usize,
  err: Option<io::Error>,
  stream: Fuse<T>,
  block_size: usize,
  exact: bool
}

impl<T> BufferedStream<T>
  where T: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  pub fn new(s: T, block_size: usize, exact: bool) -> BufferedStream<T> {
    assert!(block_size > 0);
    BufferedStream {
      items: VecDeque::new(),
      total: 0,
      err: None,
      stream: s.fuse(),
      block_size: block_size,
      exact: exact
    }
  }

  fn drain(&mut self) -> Vec<Bytes> {
    let mut rv = Vec::<Bytes>::new();
    let mut count = 0;

    while self.items.len() > 0 && count < self.block_size {
      let chunk = self.items.pop_front().unwrap();
      if (count + chunk.len() <= self.block_size) || !self.exact {
        count += chunk.len();
        self.total -= chunk.len();
        rv.push(chunk);
      } else {
        let n = self.block_size - count;
        count += n;
        self.total -= n;
        rv.push(chunk.slice(0, n));
        self.items.push_front(chunk.slice_from(n));
      }
    }

    rv
  }
}

impl<T> Stream for BufferedStream<T>
  where T: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  type Item = Vec<Bytes>;
  type Error = io::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    if let Some(err) = self.err.take() {
      return Err(err)
    }

    if self.total >= self.block_size {
      return Ok(Async::Ready(Some(self.drain())))
    }

    loop {
      match self.stream.poll() {
        Ok(Async::NotReady) => {
          return Ok(Async::NotReady);
        }

        Ok(Async::Ready(Some(item))) => {
          self.total += item.iter().fold(0, |sum, buffer| { sum + buffer.len() });
          self.items.extend(item);
          if self.total >= self.block_size {
            return Ok(Async::Ready(Some(self.drain())))
          }
          // otherwise, fall thru and try for more.
        }

        Ok(Async::Ready(None)) => {
          return Ok(Async::Ready(if self.items.len() > 0 { Some(self.drain()) } else { None }))
        }

        // mimic streams lib: send anything queued up first.
        Err(e) => {
          if self.items.len() == 0 {
            return Err(e)
          } else {
            self.err = Some(e);
            return Ok(Async::Ready(Some(self.drain())))
          }
        }
      }
    }
  }
}
