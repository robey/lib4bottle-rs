extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use bytes::Bytes;
  use lib4bottle::buffered_stream::BufferedStream;
  use lib4bottle::stream_helpers::{flatten_stream, make_stream_2, make_stream_4, pack_stream, string_stream};

  #[test]
  fn combine_small_buffers() {
    let s = make_stream_4(
      Bytes::from_static(b"hell"),
      Bytes::from_static(b"ok"),
      Bytes::from_static(b"it"),
      Bytes::from_static(b"ty!")
    );
    let b = BufferedStream::new(s, 1024, false);
    assert_eq!(string_stream(pack_stream(b)), vec![ "hellokitty!" ]);
  }

  #[test]
  fn stops_at_target() {
    let s = make_stream_4(
      Bytes::from_static(b"hell"),
      Bytes::from_static(b"ok"),
      Bytes::from_static(b"it"),
      Bytes::from_static(b"ty!")
    );
    let b = BufferedStream::new(s, 5, false);
    assert_eq!(string_stream(pack_stream(b)), vec![ "hellok", "itty!" ]);
  }

  #[test]
  fn slices_exactly() {
    let s = make_stream_2(
      Bytes::from_static(b"hell"),
      Bytes::from_static(b"okittyhowareyou!")
    );
    let b = BufferedStream::new(s, 5, true);
    assert_eq!(string_stream(pack_stream(b)), vec![ "hello", "kitty", "howar", "eyou!" ]);
  }
}
