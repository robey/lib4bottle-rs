extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_stream_generator {
  use futures::{future, Future, stream, Stream};
  use lib4bottle::stream_toolkit::generate_stream;
  use std::{io, thread, time};

  #[test]
  fn generate_small_stream() {
    let (stream, future) = generate_stream(0, |counter| {
      future::ok::<_, io::Error>(
        if counter < 10 { (Some(counter), future::ok(counter + 1)) } else { (None, future::ok(counter)) }
      )
    });

    assert_eq!(stream.collect().wait().unwrap(), vec![ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 ]);
    assert_eq!(future.wait().unwrap(), 10);
  }

  #[test]
  fn generate_nested_stream() {
    let source: Vec<Result<usize, io::Error>> = (0..10).map(|n| Ok(n)).collect();
    let (stream, future) = generate_stream(stream::iter(source), |s| {
      s.into_future().map(|(possible_n, s)| {
        let item = possible_n.and_then(|n| if n < 3 { Some(n) } else { None });
        ( item, future::ok(s) )
      }).map_err(|(e, _)| e)
    });

    assert_eq!(stream.collect().wait().unwrap(), vec![ 0, 1, 2 ]);
    assert_eq!(future.wait().unwrap().collect().wait().unwrap(), vec![ 4, 5, 6, 7, 8, 9 ]);
  }

  #[test]
  fn wake_up_future() {
    let (stream, future) = generate_stream(0, |counter| {
      future::ok::<_, io::Error>(
        if counter < 10 { (Some(counter), future::ok(counter + 1)) } else { (None, future::ok(counter)) }
      )
    });

    let t = thread::spawn(|| {
      thread::sleep(time::Duration::from_millis(50));
      assert_eq!(stream.collect().wait().unwrap(), vec![ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 ]);
    });
    assert_eq!(future.wait().unwrap(), 10);
    t.join().unwrap();
  }
}
