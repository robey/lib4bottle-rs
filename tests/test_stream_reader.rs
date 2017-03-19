extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use bytes::{Bytes};
  use futures::{Future, Stream};
  use lib4bottle::{ToHex};
  use lib4bottle::stream_helpers::{flatten_stream, make_stream_1};
  use lib4bottle::stream_reader::{stream_reader};

  #[test]
  fn read_exact_slices() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let mut r = stream_reader(s);
    assert_eq!(r.read_exact(3).wait().unwrap().to_hex(), "70726f");
    assert_eq!(r.read_exact(2).wait().unwrap().to_hex(), "6772");
    assert_eq!(r.read_exact(5).wait().unwrap().to_hex(), "6573736976");
    assert_eq!(r.read_exact(1).wait().unwrap().to_hex(), "65");
    assert!(r.read_exact(1).wait().is_err());
  }

  #[test]
  fn read_exact_refuses_to_truncate() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let mut r = stream_reader(s);
    assert!(r.read_exact(12).wait().is_err());
  }

  #[test]
  fn drain_after_read() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let mut r = stream_reader(s);
    assert_eq!(r.read_exact(3).wait().unwrap().to_hex(), "70726f");
    assert_eq!(r.into_stream().collect().wait().unwrap().to_hex(), "6772657373697665");
  }

  #[test]
  fn read() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let mut r = stream_reader(s);
    assert_eq!(r.read(3).wait().unwrap().to_hex(), "70726f");
    assert_eq!(r.read(2).wait().unwrap().to_hex(), "6772");
    assert_eq!(r.read(5).wait().unwrap().to_hex(), "6573736976");
    assert_eq!(r.read(2).wait().unwrap().to_hex(), "65");
    assert!(r.read(1).wait().is_err());
  }
}
