use bytes::Bytes;
use futures::{Future, Stream, stream};
use std::io;

use to_hex::ToHex;

pub fn make_stream_1(b1: Bytes) -> impl Stream<Item = Vec<Bytes>, Error = io::Error> {
  stream::iter(vec![ Ok(vec![ b1 ]) ])
}

pub fn make_stream_2(b1: Bytes, b2: Bytes) -> impl Stream<Item = Vec<Bytes>, Error = io::Error> {
  stream::iter(vec![ Ok(vec![ b1 ]), Ok(vec![ b2 ]) ])
}

pub fn make_stream_3(b1: Bytes, b2: Bytes, b3: Bytes) -> impl Stream<Item = Vec<Bytes>, Error = io::Error> {
  stream::iter(vec![ Ok(vec![ b1 ]), Ok(vec![ b2 ]), Ok(vec![ b3 ]) ])
}

pub fn make_stream_4(b1: Bytes, b2: Bytes, b3: Bytes, b4: Bytes) -> impl Stream<Item = Vec<Bytes>, Error = io::Error> {
  stream::iter(vec![ Ok(vec![ b1 ]), Ok(vec![ b2 ]), Ok(vec![ b3 ]), Ok(vec![ b4 ]) ])
}

// convert a stream into a vector of hex output, for tests
pub fn hex_stream<T>(s: T) -> Vec<String>
  where T: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  s.collect().wait().unwrap().iter().map(|b| b.to_hex()).collect::<Vec<String>>()
}

// convert a stream into a vector of string output, for tests
pub fn string_stream<T>(s: T) -> Vec<String>
  where T: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  s.collect().wait().unwrap().iter().map(|vec| {
    vec.iter().map(|b| String::from_utf8(b.to_vec()).unwrap()).collect::<Vec<String>>().join("")
  }).collect::<Vec<String>>()
}

// convert a stream into a single Bytes
pub fn drain_stream<T>(s: T) -> Vec<u8>
  where T: Stream<Item = Vec<Bytes>, Error = io::Error>
{
  let mut rv: Vec<u8> = Vec::new();
  for vec in s.collect().wait().unwrap() {
    for b in vec {
      rv.extend(b.to_vec())
    }
  }
  rv
}
