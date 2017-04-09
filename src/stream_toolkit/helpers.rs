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
