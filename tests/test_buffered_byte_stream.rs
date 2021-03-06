extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_buffered_stream {
  use bytes::Bytes;
  use lib4bottle::stream_toolkit::{BufferedByteStream, stream_of_vec, stream_to_string_vec};

  #[test]
  fn combine_small_buffers() {
    let s = stream_of_vec(vec![
      Bytes::from_static(b"hell"),
      Bytes::from_static(b"ok"),
      Bytes::from_static(b"it"),
      Bytes::from_static(b"ty!")
    ]);
    let b = BufferedByteStream::new(s, 1024, false);
    assert_eq!(stream_to_string_vec(b.pack()), vec![ "hellokitty!" ]);
  }

  #[test]
  fn stops_at_target() {
    let s = stream_of_vec(vec![
      Bytes::from_static(b"hell"),
      Bytes::from_static(b"ok"),
      Bytes::from_static(b"it"),
      Bytes::from_static(b"ty!")
    ]);
    let b = BufferedByteStream::new(s, 5, false);
    assert_eq!(stream_to_string_vec(b.pack()), vec![ "hellok", "itty!" ]);
  }

  #[test]
  fn slices_exactly() {
    let s = stream_of_vec(vec![
      Bytes::from_static(b"hell"),
      Bytes::from_static(b"okittyhowareyou!")
    ]);
    let b = BufferedByteStream::new(s, 5, true);
    assert_eq!(stream_to_string_vec(b.pack()), vec![ "hello", "kitty", "howar", "eyou!" ]);
  }
}
