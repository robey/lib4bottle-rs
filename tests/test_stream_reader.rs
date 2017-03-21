extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use bytes::{Bytes};
  use futures::{Future, Stream};
  use lib4bottle::{ToHex};
  use lib4bottle::stream_helpers::{make_stream, make_stream_1};
  use lib4bottle::stream_reader::{StreamReader, StreamReaderMode};

  #[test]
  fn stream_read_exact_slices() {
    let s = make_stream_1(Bytes::from_static(b"progressive"));
    let (data1, s1) = StreamReader::read_exact(s, 3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    let (data2, s2) = StreamReader::read_exact(s1, 2).wait().unwrap();
    assert_eq!(data2.vec.to_hex(), "6772");
    let (data3, s3) = StreamReader::read_exact(s2, 5).wait().unwrap();
    assert_eq!(data3.vec.to_hex(), "6573736976");
    let (data4, s4) = StreamReader::read_exact(s3, 1).wait().unwrap();
    assert_eq!(data4.vec.to_hex(), "65");
    assert!(StreamReader::read_exact(s4, 1).wait().is_err());
  }

  #[test]
  fn stream_read_exact_slices_from_chunks() {
    let s = make_stream(vec![
      Bytes::from_static(b"pr"),
      Bytes::from_static(b"ogres"),
      Bytes::from_static(b"s"),
      Bytes::from_static(b"i"),
      Bytes::from_static(b"ve")
    ]);
    let (data1, s1) = StreamReader::read_exact(s, 3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    let (data2, s2) = StreamReader::read_exact(s1, 2).wait().unwrap();
    assert_eq!(data2.vec.to_hex(), "6772");
    let (data3, s3) = StreamReader::read_exact(s2, 5).wait().unwrap();
    assert_eq!(data3.vec.to_hex(), "6573736976");
    let (data4, s4) = StreamReader::read_exact(s3, 1).wait().unwrap();
    assert_eq!(data4.vec.to_hex(), "65");
    assert!(StreamReader::read_exact(s4, 1).wait().is_err());
  }

  #[test]
  fn stream_read_exact_refuses_to_truncate() {
    let s = make_stream_1(Bytes::from_static(b"progressive"));
    assert!(StreamReader::read_exact(s, 12).wait().is_err());
  }

  #[test]
  fn stream_read_exact_returns_valid_continuation_stream() {
    let s = make_stream_1(Bytes::from_static(b"progressive"));
    let (data1, s1) = StreamReader::read_exact(s, 3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    assert_eq!(s1.collect().wait().unwrap().to_hex(), "6772657373697665");
  }

  #[test]
  fn stream_read_at_most_works() {
    let s = make_stream_1(Bytes::from_static(b"progressive"));
    let (data1, s1) = StreamReader::read_at_most(s, 3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    let (data2, s2) = StreamReader::read_at_most(s1, 2).wait().unwrap();
    assert_eq!(data2.vec.to_hex(), "6772");
    let (data3, s3) = StreamReader::read_at_most(s2, 5).wait().unwrap();
    assert_eq!(data3.vec.to_hex(), "6573736976");
    let (data4, s4) = StreamReader::read_at_most(s3, 2).wait().unwrap();
    assert_eq!(data4.vec.to_hex(), "65");
    let (data5, s5) = StreamReader::read_at_most(s4, 1).wait().unwrap();
    assert_eq!(data5.vec.to_hex(), "");
    assert_eq!(s5.collect().wait().unwrap().to_hex(), "");
  }

  #[test]
  fn stream_read_buffered_works() {
    let s = make_stream(vec![
      Bytes::from_static(b"pr"),
      Bytes::from_static(b"ogres"),
      Bytes::from_static(b"siv"),
      Bytes::from_static(b"e tran"),
      Bytes::from_static(b"ce")
    ]);
    let rv1 = StreamReader::read(s, 3, StreamReaderMode::Lazy, None).wait().unwrap();
    assert_eq!(rv1.frame.vec.to_hex(), "70726f67726573");
    let rv2 = StreamReader::read(rv1.stream, 2, StreamReaderMode::Lazy, rv1.remainder).wait().unwrap();
    assert_eq!(rv2.frame.vec.to_hex(), "736976");
    let rv3 = StreamReader::read(rv2.stream, 10, StreamReaderMode::Lazy, rv2.remainder).wait().unwrap();
    assert_eq!(rv3.frame.vec.to_hex(), "65207472616e6365");
    let rv4 = StreamReader::read(rv3.stream, 10, StreamReaderMode::Lazy, rv3.remainder).wait().unwrap();
    assert_eq!(rv4.frame.vec.to_hex(), "");
    assert_eq!(rv4.remainder, None);
    assert_eq!(rv4.stream.collect().wait().unwrap().to_hex(), "");
  }
}
