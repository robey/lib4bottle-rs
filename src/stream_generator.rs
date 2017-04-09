use futures::{Async, Future, IntoFuture, Poll, Stream, task};
use std::sync::{Arc, Mutex};

enum State<St, Fut> {
  /// No active future is running. Ready to call the function and process the next future.
  Ready(St),

  /// Currently polling this future.
  Working(Fut),

  /// Stream ended!
  Done(Option<St>),

  /// Stream ended, but not in a good way.
  Error
}

struct Inner<St, Fut: IntoFuture> {
  generator_state: Option<State<St, Fut::Future>>,
  task: Option<task::Task>
}

/// Generate a stream from an initial state, and a function that returns a
/// future for each successive item. The function is called with the state,
/// and returns a future of the next item and the next state. As each future
/// resolves, the function is called again, until it finally resolves to
/// `None`, ending the stream. At that point, the final state is resolved
/// in a separate completion future, returning the state object to the
/// caller.
///
/// This function is like `stream::unfold` in the futures library, except
/// that it returns the state to you at the end, and the decision about
/// whether the stream is complete is deferred until each future is resolved.
///
/// *Performance implications*: The state and future are stored on the heap,
/// to allow the generator to hand off the final state to the completion
/// future.
///
/// As a silly example, you can generate a stream of the first 10 ints:
///
/// ```rust,ignore
/// let (stream, completion) = generate(0, |counter| {
///   future::ok::<_, io::Error>(
///     if counter < 5 { (Some(counter), counter + 1) } else { (None, counter) }
///   )
/// });
/// ```
///
/// FIXME: say moar
pub fn generate<St, F, Fut, It>(state: St, f: F)
  -> (StreamGenerator<St, F, Fut>, StreamGeneratorCompletion<St, Fut>)
  where
    F: FnMut(St) -> Fut,
    Fut: IntoFuture<Item = (Option<It>, St)>
{
  let inner = Arc::new(Mutex::new(Inner { generator_state: Some(State::Ready(state)), task: None }));
  let generator = StreamGenerator { inner: inner.clone(), f };
  let completion = StreamGeneratorCompletion { inner: inner.clone() };
  ( generator, completion )
}

/// Like `stream::unfold`, except the function always returns a future (so
/// the future may wait until it resolves to decide if the stream has ended),
/// and the final state can be extracted from the finished stream. (FIXME)
///
/// Given an initial state `state`, and a function `f`, this struct creates
/// a stream. Each time a new item is requested from the stream, it calls
/// `f`, which returns an `(Option<It>, St)`:
///   - `None`: the stream is over
///   - `Some(item)`: emit the item and call this function again with
///     the new state to generate the next item.
pub struct StreamGenerator<St, F, Fut: IntoFuture> {
  f: F,
  inner: Arc<Mutex<Inner<St, Fut>>>
}

impl<St, F, Fut, It> Stream for StreamGenerator<St, F, Fut>
  where
    F: FnMut(St) -> Fut,
    Fut: IntoFuture<Item = (Option<It>, St)>
{
  type Item = It;
  type Error = Fut::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    let mut inner = self.inner.lock().unwrap();

    loop {
      match inner.generator_state.take().expect("polling stream twice") {
        State::Ready(state) => {
          let fut = (self.f)(state).into_future();
          inner.generator_state = Some(State::Working(fut));
        },

        State::Working(mut fut) => {
          match fut.poll() {
            Err(e) => {
              inner.generator_state = Some(State::Error);
              return Err(e);
            },
            Ok(Async::NotReady) => {
              inner.generator_state = Some(State::Working(fut));
              return Ok(Async::NotReady);
            },
            Ok(Async::Ready((None, state))) => {
              inner.generator_state = Some(State::Done(Some(state)));
              for t in inner.task.take() { t.unpark() };
              return Ok(Async::Ready(None));
            },
            Ok(Async::Ready((Some(item), state))) => {
              inner.generator_state = Some(State::Ready(state));
              return Ok(Async::Ready(Some(item)));
            }
          }
        }

        State::Done(_) => {
          return Ok(Async::Ready(None))
        },

        // it makes no sense to poll a stream after an error, so just keep saying it ended.
        State::Error => {
          return Ok(Async::Ready(None))
        }
      }
    }
  }
}


// ----- StreamGeneratorCompletion

#[must_use = "futures do nothing unless polled"]
pub struct StreamGeneratorCompletion<St, Fut: IntoFuture> {
  inner: Arc<Mutex<Inner<St, Fut>>>,
}

impl<St, Fut> Future for StreamGeneratorCompletion<St, Fut>
  where
    Fut: IntoFuture
{
  type Item = St;
  type Error = Fut::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    let mut inner = self.inner.lock().unwrap();
    match inner.generator_state.take().expect("polling future twice") {
      State::Done(Some(state)) => Ok(Async::Ready(state)),
      other => {
        inner.task = Some(task::park());
        inner.generator_state = Some(other);
        Ok(Async::NotReady)
      }
    }
  }
}
