use futures::{Async, Future, IntoFuture, Poll, Stream, task};
use std::sync::{Arc, Mutex};

pub trait StreamSplit {
  fn split_when<P, R>(self, is_last: P) -> (LeftStream<Self, P, R>, RightStream<Self, P, R>)
    where
      Self: Stream + Sized,
      P: FnMut(&Self::Item) -> R,
      R: IntoFuture<Item = bool, Error = Self::Error>;
}

impl<S> StreamSplit for S where S: Stream + Sized {
  fn split_when<P, R>(self, is_last: P) -> (LeftStream<Self, P, R>, RightStream<Self, P, R>)
    where
      P: FnMut(&S::Item) -> R,
      R: IntoFuture<Item = bool, Error = S::Error>
  {
    let inner = Arc::new(Mutex::new(Inner::new(self, is_last)));
    ( LeftStream { inner: inner.clone() }, RightStream { inner: inner.clone() } )
  }
}


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
  is_last: P,

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
  fn new(stream: S, is_last: P) -> Inner<S, P, R> {
    Inner {
      state: State::Left,
      stream,
      is_last,
      pending_future: None,
      pending_item: None,
      right_task: None
    }
  }

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
        self.pending_future = Some((self.is_last)(&item).into_future());
        self.pending_item = Some(item);
        Ok(Async::Ready(true))
      },
    }
  }

  fn clear_pending(&mut self) {
    self.pending_future = None;
    self.pending_item = None;
  }

  // switch from left mode to right mode.
  fn transition(&mut self) {
    self.clear_pending();
    self.state = State::Right;
    for t in self.right_task.take() { t.unpark() };
  }
}


// ----- LeftStream

#[must_use = "streams do nothing unless polled"]
pub struct LeftStream<S, P, R> where S: Stream, R: IntoFuture {
  inner: Arc<Mutex<Inner<S, P, R>>>,
}

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
          Ok(Async::Ready(last)) => {
            let item = inner.pending_item.take().unwrap();
            if last {
              inner.transition();
            } else {
              inner.clear_pending();
            }
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

    inner.stream.poll()
  }

  // fn into_inner(self) -> S {
  //   let mut inner = self.inner.lock().unwrap();
  //   inner.stream
  // }
}
