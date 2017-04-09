extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_stream_reader {
  use bytes::{Bytes};
  use futures::{Future, Stream};
  use lib4bottle::hex::{ToHex};
  use lib4bottle::stream_helpers::{stream_of, stream_of_vec};
  use lib4bottle::stream_toolkit::{ReadableByteStream, ReadMode};

  #[test]
  fn stream_read_exact_slices() {
    let s = ReadableByteStream::from(stream_of(Bytes::from_static(b"progressive")));
    let (data1, s) = s.read_exact(3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    let (data2, s) = s.read_exact(2).wait().unwrap();
    assert_eq!(data2.vec.to_hex(), "6772");
    let (data3, s) = s.read_exact(5).wait().unwrap();
    assert_eq!(data3.vec.to_hex(), "6573736976");
    let (data4, s) = s.read_exact(1).wait().unwrap();
    assert_eq!(data4.vec.to_hex(), "65");
    assert!(s.read_exact(1).wait().is_err());
  }

  #[test]
  fn stream_read_exact_slices_from_chunks() {
    let s = ReadableByteStream::from(stream_of_vec(vec![
      Bytes::from_static(b"pr"),
      Bytes::from_static(b"ogres"),
      Bytes::from_static(b"s"),
      Bytes::from_static(b"i"),
      Bytes::from_static(b"ve")
    ]));
    let (data1, s) = s.read_exact(3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    let (data2, s) = s.read_exact(2).wait().unwrap();
    assert_eq!(data2.vec.to_hex(), "6772");
    let (data3, s) = s.read_exact(5).wait().unwrap();
    assert_eq!(data3.vec.to_hex(), "6573736976");
    let (data4, s) = s.read_exact(1).wait().unwrap();
    assert_eq!(data4.vec.to_hex(), "65");
    assert!(s.read_exact(1).wait().is_err());
  }

  #[test]
  fn stream_read_exact_refuses_to_truncate() {
    let s = ReadableByteStream::from(stream_of(Bytes::from_static(b"progressive")));
    assert!(s.read_exact(12).wait().is_err());
  }

  #[test]
  fn stream_read_exact_returns_valid_continuation_stream() {
    let s = ReadableByteStream::from(stream_of(Bytes::from_static(b"progressive")));
    let (data1, s) = s.read_exact(3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    assert_eq!(s.into_stream().collect().wait().unwrap().to_hex(), "6772657373697665");
  }

  #[test]
  fn stream_read_at_most_works() {
    let s = ReadableByteStream::from(stream_of(Bytes::from_static(b"progressive")));
    let (data1, s) = s.read_at_most(3).wait().unwrap();
    assert_eq!(data1.vec.to_hex(), "70726f");
    let (data2, s) = s.read_at_most(2).wait().unwrap();
    assert_eq!(data2.vec.to_hex(), "6772");
    let (data3, s) = s.read_at_most(5).wait().unwrap();
    assert_eq!(data3.vec.to_hex(), "6573736976");
    let (data4, s) = s.read_at_most(2).wait().unwrap();
    assert_eq!(data4.vec.to_hex(), "65");
    let (data5, s) = s.read_at_most(1).wait().unwrap();
    assert_eq!(data5.vec.to_hex(), "");
    assert_eq!(s.into_stream().collect().wait().unwrap().to_hex(), "");
  }

  #[test]
  fn stream_read_buffered_works() {
    let s = ReadableByteStream::from(stream_of_vec(vec![
      Bytes::from_static(b"pr"),
      Bytes::from_static(b"ogres"),
      Bytes::from_static(b"siv"),
      Bytes::from_static(b"e tran"),
      Bytes::from_static(b"ce")
    ]));
    let (frame1, s) = s.read(3, ReadMode::Lazy).wait().unwrap();
    assert_eq!(frame1.vec.to_hex(), "70726f67726573");
    let (frame2, s) = s.read(2, ReadMode::Lazy).wait().unwrap();
    assert_eq!(frame2.vec.to_hex(), "736976");
    let (frame3, s) = s.read(10, ReadMode::Lazy).wait().unwrap();
    assert_eq!(frame3.vec.to_hex(), "65207472616e6365");
    let (frame4, s) = s.read(10, ReadMode::Lazy).wait().unwrap();
    assert_eq!(frame4.vec.to_hex(), "");
    assert_eq!(s.into_stream().collect().wait().unwrap().to_hex(), "");
  }
}
