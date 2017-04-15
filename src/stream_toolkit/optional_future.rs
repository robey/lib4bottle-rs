use futures::{Async, Future, Poll};

/// Convert an Option<Future<T>> into a Future<Option<T>>.
/// A clever variant of this is in the futures library, but for some reason,
/// it doesn't work. I think maybe converting the Option directly to a
/// Future may be a little *too* clever for Rust's type-checker, possibly
/// made worse by both types having many of the same functions.
pub struct OptionFuture<F, I, E>
  where F: Future<Item = I, Error = E>
{
  inner: Option<F>,
}

pub trait OptionToFuture<F, I, E>
  where F: Future<Item = I, Error = E>
{
  // can't call it into_future, because of the existing implicit in futures-rs
  fn to_future(self) -> OptionFuture<F, I, E>;
}

impl<F, I, E> OptionToFuture<F, I, E> for Option<F>
  where F: Future<Item = I, Error = E>
{
  fn to_future(self) -> OptionFuture<F, I, E> {
    OptionFuture { inner: self }
  }
}

impl<F, I, E> Future for OptionFuture<F, I, E>
  where F: Future<Item = I, Error = E>
{
  type Item = Option<I>;
  type Error = E;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    match self.inner {
      None => Ok(Async::Ready(None)),
      Some(ref mut future) => future.poll().map(|async| async.map(Some))
    }
  }
}
