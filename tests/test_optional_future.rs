extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_optional_future {
  use futures::{Future, future};

  #[test]
  fn optional_none() {
    let f: Option<future::FutureResult<u32, u32>> = None;
    assert_eq!(f.wait().unwrap(), None);
  }

  #[test]
  fn optional_some() {
    let f: Option<future::FutureResult<u32, u32>> = Some(future::ok(10));
    assert_eq!(f.wait().unwrap(), Some(10));
  }
}
