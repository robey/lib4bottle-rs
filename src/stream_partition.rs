use bytes::Bytes;
use futures::{Async, Future, IntoFuture, Poll, Stream, task};
use std::io;
use std::sync::{Arc, Mutex};

// left state:
//   - left stream: normal
//   - right stream: park, not_ready
// transition:
//   - left stream:
//       - save popped item
//       - switch state
//       - unpark right stream
//       - ready(none)
// right state:
//   - left stream: ready(none)
//   - right stream: any popped item, then normal

/// Which stream is currently "active"?
#[derive(Clone, PartialEq)]
enum State {
  Left,
  Right
}

// data shared by both left & right
struct Inner<S, P, R> where S: Stream, R: IntoFuture {
  state: State,
  stream: S,
  predicate: P,

  // when we have an item, but we're waiting for the future to complete:
  pending_future: Option<R::Future>,
  pending_item: Option<S::Item>,

  // when someone tried to read the right stream before it started:
  right_task: Option<task::Task>,
}

impl<S, P, R> Inner<S, P, R>
  where
    S: Stream,
    P: FnMut(&S::Item) -> R,
    R: IntoFuture<Item = bool, Error = S::Error>
{
  fn poll_pending(&mut self) -> Poll<bool, S::Error> {
    if self.pending_future.is_some() {
      return Ok(Async::Ready(true));
    }

    match self.stream.poll() {
      Err(e) => {
        self.clear_pending();
        Err(e)
      },
      Ok(Async::NotReady) => Ok(Async::NotReady),
      Ok(Async::Ready(None)) => Ok(Async::Ready(false)),
      Ok(Async::Ready(Some(item))) => {
        self.pending_future = Some((self.predicate)(&item).into_future());
        self.pending_item = Some(item);
        Ok(Async::Ready(true))
      },
    }
  }

  fn clear_pending(&mut self) {
    self.pending_future = None;
    self.pending_item = None;
  }

}


// ----- LeftStream

#[must_use = "streams do nothing unless polled"]
pub struct LeftStream<S, P, R> where S: Stream, R: IntoFuture {
  inner: Arc<Mutex<Inner<S, P, R>>>,
}

// impl<S, P, R> StreamPartition<S, P, R> where S: Stream, R: IntoFuture {
//   pub fn new(s: S, p: P) -> StreamPartition<S, P, R> where P: FnMut(&S::Item) -> R, R: IntoFuture<Item = bool, Error = S::Error> {
//     StreamPartition {
//       inner: Box::new(Inner { stream: s, pred: p }),
//       pred: p,
//       first_stream: s.take_while(p).boxed()
//     }
//   }
// }

impl<S, P, R> Stream for LeftStream<S, P, R>
  where
    S: Stream,
    P: FnMut(&S::Item) -> R,
    R: IntoFuture<Item = bool, Error = S::Error>
{
  type Item = S::Item;
  type Error = S::Error;

  fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
    let mut inner = self.inner.lock().unwrap();
    if inner.state == State::Right {
      return Ok(Async::Ready(None));
    }

    match inner.poll_pending() {
      Err(e) => Err(e),
      Ok(Async::NotReady) => Ok(Async::NotReady),
      Ok(Async::Ready(false)) => {
        // end of stream: transition anyway.
        inner.state = State::Right;
        for t in inner.right_task.take() { t.unpark() };
        Ok(Async::Ready(None))
      },
      Ok(Async::Ready(true)) => {
        match inner.pending_future.as_mut().unwrap().poll() {
          Err(e) => {
            inner.clear_pending();
            Err(e)
          },
          Ok(Async::NotReady) => Ok(Async::NotReady),
          Ok(Async::Ready(false)) => {
            // transition:
            inner.pending_future = None;
            inner.state = State::Right;
            for t in inner.right_task.take() { t.unpark() };
            Ok(Async::Ready(None))
          },
          Ok(Async::Ready(true)) => {
            let item = inner.pending_item.take().unwrap();
            inner.clear_pending();
            Ok(Async::Ready(Some(item)))
          }
        }
      }
    }
  }
}


// ----- RightStream

#[must_use = "streams do nothing unless polled"]
pub struct RightStream<S, P, R> where S: Stream, R: IntoFuture {
  inner: Arc<Mutex<Inner<S, P, R>>>,
}

impl<S, P, R> Stream for RightStream<S, P, R>
  where
    S: Stream,
    P: FnMut(&S::Item) -> R,
    R: IntoFuture<Item = bool, Error = S::Error>
{
  type Item = S::Item;
  type Error = S::Error;

  fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
    let mut inner = self.inner.lock().unwrap();
    if inner.state == State::Left {
      inner.right_task = Some(task::park());
      return Ok(Async::NotReady);
    }

    if inner.pending_item.is_some() {
      // leftover item that failed the predicate, causing us to switch to this state.
      let item = inner.pending_item.take().unwrap();
      return Ok(Async::Ready(Some(item)));
    }

    inner.stream.poll()
  }
}


// ----- Inner


pub struct SharedStream<S: Stream> {
  inner: Arc<Mutex<S>>
}

impl<S> SharedStream<S>
  where S: Stream
{
  pub fn clone(&self) -> SharedStream<S> {
    SharedStream { inner: self.inner.clone() }
  }
}

pub fn robey<S>(s: S) -> SharedStream<S>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  SharedStream { inner: Arc::new(Mutex::new(s)) }
}
