use futures::{Async, Future, IntoFuture, Poll, Stream, task};
use std::sync::{Arc, Mutex};

pub trait SplitUntil {
  /// Split this stream in two, using a function to determine which item is
  /// the split point.
  ///
  /// Returns a `SplitStream` representing the stream up to the split point,
  /// and a `SplitFuture` which will resolve to the original stream once the
  /// `SplitStream` has been drained.
  ///
  /// As items arrive, they are passed to `is_last`, which will return a
  /// `Future<bool>` to determine if the split stream should end after this
  /// item. Each item will be fed into `SplitStream` after `is_last`
  /// resolves. If `is_last` resolves to true, the `SplitStream` is
  /// completed, and `SplitFuture` resolves to the remainder of the original
  /// stream.
  fn split_until<P, R>(self, is_last: P) -> (SplitStream<Self, P, R>, SplitFuture<Self, P, R>)
    where
      Self: Stream + Sized,
      P: FnMut(&Self::Item) -> R,
      R: IntoFuture<Item = bool, Error = Self::Error>;
}

impl<S> SplitUntil for S where S: Stream + Sized {
  fn split_until<P, R>(self, is_last: P) -> (SplitStream<Self, P, R>, SplitFuture<Self, P, R>)
    where
      P: FnMut(&S::Item) -> R,
      R: IntoFuture<Item = bool, Error = S::Error>
  {
    let inner = Arc::new(Mutex::new(Inner::new(self, is_last)));
    ( SplitStream { inner: inner.clone() }, SplitFuture { inner: inner.clone() } )
  }
}

enum FeederState {
  // waiting for the stream to produce an item:
  Waiting,

  // the stream has ended:
  Finished,

  // an item has been fed to `is_last`:
  Processing
}

// data shared by both SplitStream & SplitFuture
struct Inner<S, P, R> where S: Stream, R: IntoFuture {
  stream: Option<S>,
  is_last: P,
  complete: bool,

  // when we have an item, but we're waiting for the future to complete:
  pending_future: Option<R::Future>,
  pending_item: Option<S::Item>,

  // when someone tried to read the right stream before it started:
  remainder_task: Option<task::Task>,
}

impl<S, P, R> Inner<S, P, R>
  where
    S: Stream,
    P: FnMut(&S::Item) -> R,
    R: IntoFuture<Item = bool, Error = S::Error>
{
  fn new(stream: S, is_last: P) -> Inner<S, P, R> {
    Inner {
      stream: Some(stream),
      is_last,
      complete: false,
      pending_future: None,
      pending_item: None,
      remainder_task: None
    }
  }

  // poll the stream for another item. if an item is ready, feed it to the
  // `is_last` function.
  fn feed(&mut self) -> Result<FeederState, S::Error> {
    if self.pending_future.is_some() {
      return Ok(FeederState::Processing);
    }

    match self.stream.as_mut().expect("stream in use").poll() {
      Err(e) => Err(e),
      Ok(Async::NotReady) => Ok(FeederState::Waiting),
      Ok(Async::Ready(None)) => Ok(FeederState::Finished),
      Ok(Async::Ready(Some(item))) => {
        self.pending_future = Some((self.is_last)(&item).into_future());
        self.pending_item = Some(item);
        Ok(FeederState::Processing)
      },
    }
  }

  fn clear_pending(&mut self) {
    self.pending_future = None;
    self.pending_item = None;
  }

  // the SplitStream has finished. tell SplitFuture, if necessary.
  fn finish(&mut self) {
    self.clear_pending();
    self.complete = true;
    for t in self.remainder_task.take() { t.unpark() };
  }
}


// ----- SplitStream

#[must_use = "streams do nothing unless polled"]
pub struct SplitStream<S, P, R> where S: Stream, R: IntoFuture {
  inner: Arc<Mutex<Inner<S, P, R>>>,
}

impl<S, P, R> Stream for SplitStream<S, P, R>
  where
    S: Stream,
    P: FnMut(&S::Item) -> R,
    R: IntoFuture<Item = bool, Error = S::Error>
{
  type Item = S::Item;
  type Error = S::Error;

  fn poll(&mut self) -> Poll<Option<S::Item>, S::Error> {
    let mut inner = self.inner.lock().unwrap();
    if inner.complete {
      return Ok(Async::Ready(None));
    }

    match inner.feed() {
      Err(e) => {
        inner.clear_pending();
        Err(e)
      },
      Ok(FeederState::Waiting) => Ok(Async::NotReady),
      Ok(FeederState::Finished) => {
        // end of stream.
        inner.finish();
        Ok(Async::Ready(None))
      },
      Ok(FeederState::Processing) => {
        match inner.pending_future.as_mut().unwrap().poll() {
          Err(e) => {
            inner.clear_pending();
            Err(e)
          },
          Ok(Async::NotReady) => Ok(Async::NotReady),
          Ok(Async::Ready(last)) => {
            let item = inner.pending_item.take().unwrap();
            if last {
              inner.finish();
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

#[must_use = "futures do nothing unless polled"]
pub struct SplitFuture<S, P, R> where S: Stream, R: IntoFuture {
  inner: Arc<Mutex<Inner<S, P, R>>>,
}

impl<S, P, R> Future for SplitFuture<S, P, R>
  where
    S: Stream,
    P: FnMut(&S::Item) -> R,
    R: IntoFuture<Item = bool, Error = S::Error>
{
  type Item = S;
  type Error = S::Error;

  fn poll(&mut self) -> Poll<S, S::Error> {
    let mut inner = self.inner.lock().unwrap();
    if !inner.complete {
      inner.remainder_task = Some(task::park());
      return Ok(Async::NotReady);
    }

    Ok(Async::Ready(inner.stream.take().expect("stream in use")))
  }
}
