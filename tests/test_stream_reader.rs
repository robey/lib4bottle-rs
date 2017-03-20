extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use bytes::{Bytes};
  use futures::{Future, Stream};
  use lib4bottle::{ToHex};
  use lib4bottle::stream_helpers::{flatten_stream, make_stream, make_stream_1};
  use lib4bottle::stream_reader::{stream_read, stream_read_buffered, stream_read_exact};

  #[test]
  fn stream_read_exact_slices() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let (data1, s1) = stream_read_exact(s, 3).wait().unwrap();
    assert_eq!(data1.to_hex(), "70726f");
    let (data2, s2) = stream_read_exact(s1, 2).wait().unwrap();
    assert_eq!(data2.to_hex(), "6772");
    let (data3, s3) = stream_read_exact(s2, 5).wait().unwrap();
    assert_eq!(data3.to_hex(), "6573736976");
    let (data4, s4) = stream_read_exact(s3, 1).wait().unwrap();
    assert_eq!(data4.to_hex(), "65");
    assert!(stream_read_exact(s4, 1).wait().is_err());
  }

  #[test]
  fn stream_read_exact_slices_from_chunks() {
    let s = flatten_stream(make_stream(vec![
      Bytes::from_static(b"pr"),
      Bytes::from_static(b"ogres"),
      Bytes::from_static(b"s"),
      Bytes::from_static(b"i"),
      Bytes::from_static(b"ve")
    ]));
    let (data1, s1) = stream_read_exact(s, 3).wait().unwrap();
    assert_eq!(data1.to_hex(), "70726f");
    let (data2, s2) = stream_read_exact(s1, 2).wait().unwrap();
    assert_eq!(data2.to_hex(), "6772");
    let (data3, s3) = stream_read_exact(s2, 5).wait().unwrap();
    assert_eq!(data3.to_hex(), "6573736976");
    let (data4, s4) = stream_read_exact(s3, 1).wait().unwrap();
    assert_eq!(data4.to_hex(), "65");
    assert!(stream_read_exact(s4, 1).wait().is_err());
  }

  #[test]
  fn stream_read_exact_refuses_to_truncate() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    assert!(stream_read_exact(s, 12).wait().is_err());
  }

  #[test]
  fn stream_read_exact_returns_valid_continuation_stream() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let (data1, s1) = stream_read_exact(s, 3).wait().unwrap();
    assert_eq!(data1.to_hex(), "70726f");
    assert_eq!(s1.collect().wait().unwrap().to_hex(), "6772657373697665");
  }

  #[test]
  fn stream_read_works() {
    let s = flatten_stream(make_stream_1(Bytes::from_static(b"progressive")));
    let (data1, s1) = stream_read(s, 3).wait().unwrap();
    assert_eq!(data1.to_hex(), "70726f");
    let (data2, s2) = stream_read(s1, 2).wait().unwrap();
    assert_eq!(data2.to_hex(), "6772");
    let (data3, s3) = stream_read(s2, 5).wait().unwrap();
    assert_eq!(data3.to_hex(), "6573736976");
    let (data4, s4) = stream_read(s3, 2).wait().unwrap();
    assert_eq!(data4.to_hex(), "65");
    let (data5, s5) = stream_read(s4, 1).wait().unwrap();
    assert_eq!(data5.to_hex(), "");
    assert_eq!(s5.collect().wait().unwrap().to_hex(), "");
  }

  #[test]
  fn stream_read_buffered_works() {
    let s = flatten_stream(make_stream(vec![
      Bytes::from_static(b"pr"),
      Bytes::from_static(b"ogres"),
      Bytes::from_static(b"siv"),
      Bytes::from_static(b"e tran"),
      Bytes::from_static(b"ce")
    ]));
    let (data1, s1) = stream_read_buffered(s, 3).wait().unwrap();
    assert_eq!(data1.to_hex(), "70726f67726573");
    let (data2, s2) = stream_read_buffered(s1, 2).wait().unwrap();
    assert_eq!(data2.to_hex(), "736976");
    let (data3, s3) = stream_read_buffered(s2, 10).wait().unwrap();
    assert_eq!(data3.to_hex(), "65207472616e6365");
    let (data4, s4) = stream_read_buffered(s3, 10).wait().unwrap();
    assert_eq!(data4.to_hex(), "");
    assert_eq!(s4.collect().wait().unwrap().to_hex(), "");
  }
}
