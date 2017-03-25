use bytes::Bytes;
use futures::{Future, Stream, stream};
use std::io;

use hex::ToHex;

pub fn stream_to_string_vec<S>(s: S) -> Vec<String> where S: Stream<Item = Bytes, Error = io::Error> {
  s.collect().wait().unwrap().iter().map(|b| {
    String::from_utf8(b.to_vec()).unwrap()
  }).collect()
}

pub fn stream_to_hex_vec<S>(s: S) -> Vec<String> where S: Stream<Item = Bytes, Error = io::Error> {
  s.collect().wait().unwrap().iter().map(|b| b.to_hex()).collect()
}

pub fn stream_of(b1: Bytes) -> impl Stream<Item = Bytes, Error = io::Error> {
  stream::once(Ok(b1))
}

pub fn stream_of_vec(v: Vec<Bytes>) -> impl Stream<Item = Bytes, Error = io::Error> {
  stream::iter(v.into_iter().map(|b| Ok(b)))
}

pub fn stream_of_streams<S>(v: Vec<S>) -> impl Stream<Item = S, Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  stream::iter(v.into_iter().map(|b| Ok(b)))
}


// pub fn make_framed_stream_1(b1: Bytes) -> impl Stream<Item = ByteFrame, Error = io::Error> {
//   let length = b1.len();
//   stream::iter(vec![ Ok(ByteFrame::new(vec![ b1 ], length)) ])
// }
//
// pub fn make_framed_stream_3(b1: Bytes, b2: Bytes, b3: Bytes) -> impl Stream<Item = ByteFrame, Error = io::Error> {
//   let length = b1.len() + b2.len() + b3.len();
//   stream::iter(vec![ Ok(ByteFrame::new(vec![ b1, b2, b3 ], length)) ])
// }
//
// pub fn make_stream_1(b1: Bytes) -> impl Stream<Item = Bytes, Error = io::Error> {
//   stream::iter(vec![ Ok(b1) ])
// }
//
// pub fn make_stream_2(b1: Bytes, b2: Bytes) -> impl Stream<Item = Bytes, Error = io::Error> {
//   stream::iter(vec![ Ok(b1), Ok(b2) ])
// }
//
// // pub fn make_stream_3(b1: Bytes, b2: Bytes, b3: Bytes) -> impl Stream<Item = Vec<Bytes>, Error = io::Error> {
// //   stream::iter(vec![ Ok(vec![ b1 ]), Ok(vec![ b2 ]), Ok(vec![ b3 ]) ])
// // }
//
// pub fn make_stream_4(b1: Bytes, b2: Bytes, b3: Bytes, b4: Bytes) -> impl Stream<Item = Bytes, Error = io::Error> {
//   stream::iter(vec![ Ok(b1), Ok(b2), Ok(b3), Ok(b4) ])
// }

// // convert a stream into a single buffer (for tests)
// pub fn drain_stream<S>(s: S) -> Vec<u8>
//   where S: Stream<Item = Vec<Bytes>, Error = io::Error>
// {
//   let mut rv: Vec<u8> = Vec::new();
//   for vec in s.collect().wait().unwrap() {
//     for b in vec {
//       rv.extend(b.to_vec())
//     }
//   }
//   rv
// }
//
// /// Convert a stream of `ByteFrame` into a stream of `Bytes` without copying.
// pub fn flatten_stream<S>(s: S) -> impl Stream<Item = Bytes, Error = io::Error>
//   where S: Stream<Item = ByteFrame, Error = io::Error>
// {
//   s.map(|frame| stream::iter(frame.vec.into_iter().map(|b| Ok(b)))).flatten()
// }
//
