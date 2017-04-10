use futures::{Async, Future, IntoFuture, Poll, Stream, task};
use std::sync::{Arc, Mutex};

enum State<StateT, ItemFutureT, StateFutureT> {
  /// No active future is running. Ready to call the function and process the next future.
  Ready(StateT),

  /// Currently polling the future to reveal the next item.
  WorkingOnItem(ItemFutureT),

  /// Currently polling the future to reveal the next state.
  WorkingOnState(StateFutureT),

  /// Stream ended!
  Done(StateFutureT),

  /// Stream ended, but not in a good way.
  Error
}

struct Inner<StateT, ItemFutureT: IntoFuture, StateFutureT: IntoFuture> {
  generator_state: Option<State<StateT, ItemFutureT::Future, StateFutureT::Future>>,
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
/// let (stream, completion) = generate_stream(0, |counter| {
///   future::ok::<_, io::Error>(
///     if counter < 5 { (Some(counter), future::ok(counter + 1)) } else { (None, future::ok(counter)) }
///   )
/// });
/// ```
///
/// FIXME: say moar
pub fn generate_stream<FunctionT, ItemFutureT, ItemT, StateFutureT, StateT, ErrorT>(state: StateT, f: FunctionT)
  -> (
    StreamGenerator<StateT, FunctionT, ItemFutureT, StateFutureT>,
    StreamGeneratorCompletion<StateT, ItemFutureT, StateFutureT>
  )
  where
    FunctionT: FnMut(StateT) -> ItemFutureT,
    ItemFutureT: IntoFuture<Item = (Option<ItemT>, StateFutureT), Error = ErrorT>,
    StateFutureT: IntoFuture<Item = StateT, Error = ErrorT>
{
  let inner = Arc::new(Mutex::new(Inner { generator_state: Some(State::Ready(state)), task: None }));
  let generator = StreamGenerator { inner: inner.clone(), f };
  let completion = StreamGeneratorCompletion { inner: inner.clone() };
  ( generator, completion )
}

pub struct StreamGenerator<StateT, FunctionT, ItemFutureT: IntoFuture, StateFutureT: IntoFuture> {
  f: FunctionT,
  inner: Arc<Mutex<Inner<StateT, ItemFutureT, StateFutureT>>>
}

impl<FunctionT, ItemFutureT, ItemT, StateFutureT, StateT, ErrorT> Stream
  for StreamGenerator<StateT, FunctionT, ItemFutureT, StateFutureT>
  where
    FunctionT: FnMut(StateT) -> ItemFutureT,
    ItemFutureT: IntoFuture<Item = (Option<ItemT>, StateFutureT), Error = ErrorT>,
    StateFutureT: IntoFuture<Item = StateT, Error = ErrorT>
{
  type Item = ItemT;
  type Error = ItemFutureT::Error;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    let mut inner = self.inner.lock().unwrap();

    loop {
      match inner.generator_state.take().expect("polling stream twice") {
        State::Ready(state) => {
          let item_future = (self.f)(state).into_future();
          inner.generator_state = Some(State::WorkingOnItem(item_future));
        },

        State::WorkingOnItem(mut item_future) => {
          return match item_future.poll() {
            Err(e) => {
              inner.generator_state = Some(State::Error);
              Err(e)
            },
            Ok(Async::NotReady) => {
              inner.generator_state = Some(State::WorkingOnItem(item_future));
              Ok(Async::NotReady)
            },
            Ok(Async::Ready((None, state_future))) => {
              inner.generator_state = Some(State::Done(state_future.into_future()));
              for t in inner.task.take() { t.unpark() };
              Ok(Async::Ready(None))
            },
            Ok(Async::Ready((Some(item), state_future))) => {
              inner.generator_state = Some(State::WorkingOnState(state_future.into_future()));
              Ok(Async::Ready(Some(item)))
            }
          };
        },

        State::WorkingOnState(mut state_future) => {
          match state_future.poll() {
            Err(e) => {
              inner.generator_state = Some(State::Error);
              return Err(e);
            },
            Ok(Async::NotReady) => {
              inner.generator_state = Some(State::WorkingOnState(state_future));
              return Ok(Async::NotReady);
            },
            Ok(Async::Ready(state)) => {
              inner.generator_state = Some(State::Ready(state));
            }
          }
        },

        State::Done(_) => {
          return Ok(Async::Ready(None));
        },

        // it makes no sense to poll a stream after an error, so just keep saying it ended.
        State::Error => {
          return Ok(Async::Ready(None));
        }
      }
    }
  }
}


// ----- StreamGeneratorCompletion

#[must_use = "futures do nothing unless polled"]
pub struct StreamGeneratorCompletion<StateT, ItemFutureT: IntoFuture, StateFutureT: IntoFuture> {
  inner: Arc<Mutex<Inner<StateT, ItemFutureT, StateFutureT>>>
}

impl<StateT, ItemFutureT, StateFutureT> Future for StreamGeneratorCompletion<StateT, ItemFutureT, StateFutureT>
  where
    ItemFutureT: IntoFuture,
    StateFutureT: IntoFuture<Item = StateT>
{
  type Item = StateT;
  type Error = StateFutureT::Error;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    let mut inner = self.inner.lock().unwrap();
    match inner.generator_state.take().expect("polling future twice") {
      State::Done(mut state_future) => {
        match state_future.poll() {
          Err(e) => Err(e),
          Ok(Async::NotReady) => {
            inner.generator_state = Some(State::Done(state_future));
            Ok(Async::NotReady)
          },
          Ok(Async::Ready(state)) => Ok(Async::Ready(state))
        }
      },
      other => {
        inner.task = Some(task::park());
        inner.generator_state = Some(other);
        Ok(Async::NotReady)
      }
    }
  }
}
