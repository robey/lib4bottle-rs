use bytes::{Bytes};
use futures::{Stream, stream};

/// A "frame" of bytes, consisting of a vector of `Bytes` objects and a
/// pre-calculated count of the total size.
pub struct ByteFrame {
  pub vec: Vec<Bytes>,
  pub length: usize
}

impl ByteFrame {
  pub fn new(vec: Vec<Bytes>, length: usize) -> ByteFrame {
    ByteFrame { vec: vec, length: length }
  }

  /// Convert a `Vec<Bytes>` into a `Bytes`, with copying. ☹️
  pub fn pack(&self) -> Bytes {
    if self.vec.len() == 1 {
      return self.vec[0].clone();
    }
    let len = self.vec.iter().fold(0, |sum, b| { sum + b.len() });
    let mut rv: Vec<u8> = Vec::with_capacity(len);
    for ref b in &self.vec { rv.extend(b.as_ref()) };
    Bytes::from(rv)
  }

  /// Convert a stream of `ByteFrame` into a stream of `Bytes` _without_ copying. 🎉
  pub fn flatten_stream<S, E>(s: S) -> impl Stream<Item = Bytes, Error = E>
    where S: Stream<Item = ByteFrame, Error = E>
  {
    s.map(|frame| stream::iter(frame.vec.into_iter().map(|b| Ok(b)))).flatten()
  }
}

impl From<Vec<Bytes>> for ByteFrame {
  fn from(v: Vec<Bytes>) -> ByteFrame {
    let length = v.iter().fold(0, |sum, b| sum + b.len());
    ByteFrame::new(v, length)
  }
}

impl From<Bytes> for ByteFrame {
  fn from(b: Bytes) -> ByteFrame {
    let length = b.len();
    ByteFrame::new(vec![ b ], length)
  }
}
