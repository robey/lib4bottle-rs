#![feature(conservative_impl_trait)]

extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_stream_split {
  use futures::{future, Future, Stream, stream};
  use std::{io, thread, time};
  use lib4bottle::stream_toolkit::{SplitUntil};

  #[test]
  fn simple_split() {
    let s = stream::iter::<_, _, io::Error>(vec![ 1, 2, 3, 4, 5, 6 ].into_iter().map(|n| Ok(n)));
    let (left, right) = s.split_until(|n| { future::ok(*n == 4) });
    assert_eq!(left.collect().wait().unwrap(), vec![ 1, 2, 3, 4 ]);
    assert_eq!(right.wait().unwrap().collect().wait().unwrap(), vec![ 5, 6 ]);
  }

  #[test]
  fn all_left() {
    let s = stream::iter::<_, _, io::Error>(vec![ 1, 2, 3, 4, 5, 6 ].into_iter().map(|n| Ok(n)));
    let (left, right) = s.split_until(|_| { future::ok(false) });
    assert_eq!(left.collect().wait().unwrap(), vec![ 1, 2, 3, 4, 5, 6 ]);
    assert_eq!(right.wait().unwrap().collect().wait().unwrap(), vec![]);
  }

  #[test]
  fn all_right() {
    let s = stream::iter::<_, _, io::Error>(vec![ 1, 2, 3, 4, 5, 6 ].into_iter().map(|n| Ok(n)));
    let (left, right) = s.split_until(|_| { future::ok(true) });
    assert_eq!(left.collect().wait().unwrap(), vec![ 1 ]);
    assert_eq!(right.wait().unwrap().collect().wait().unwrap(), vec![ 2, 3, 4, 5, 6 ]);
  }

  #[test]
  fn wake_up_right_stream() {
    let s = stream::iter::<_, _, io::Error>(vec![ 1, 2, 3, 4, 5, 6 ].into_iter().map(|n| Ok(n)));
    let (left, right) = s.split_until(|n| { future::ok(*n == 4) });
    let t = thread::spawn(|| {
      thread::sleep(time::Duration::from_millis(50));
      assert_eq!(left.collect().wait().unwrap(), vec![ 1, 2, 3, 4 ]);
    });
    assert_eq!(right.wait().unwrap().collect().wait().unwrap(), vec![ 5, 6 ]);
    t.join().unwrap();
  }
}
