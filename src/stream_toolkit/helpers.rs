use bytes::Bytes;
use futures::{Future, Stream, stream};
use std::io;

use super::{FromHex, ReadableByteStream, ToHex};

pub fn stream_to_string_vec<S>(s: S) -> Vec<String> where S: Stream<Item = Bytes, Error = io::Error> {
  s.collect().wait().unwrap().iter().map(|b| {
    String::from_utf8(b.to_vec()).unwrap()
  }).collect()
}

pub fn stream_to_hex_vec<S>(s: S) -> Vec<String> where S: Stream<Item = Bytes, Error = io::Error> {
  s.collect().wait().unwrap().iter().map(|b| b.to_hex()).collect()
}

/// Generate a `Stream<Bytes>` from a single `Bytes` object.
pub fn stream_of(b1: Bytes) -> impl Stream<Item = Bytes, Error = io::Error> {
  stream::once(Ok(b1))
}

/// Generate a `Stream<Bytes>` from a sequence of `Bytes` objects (any iterable).
pub fn stream_of_vec<I: IntoIterator<Item = Bytes>>(vec: I) -> impl Stream<Item = Bytes, Error = io::Error> {
  stream::iter(vec.into_iter().map(|b| Ok(b)))
}

/// Generate a `Stream<Stream<Bytes>>` from a sequence of `Stream<Bytes>` objects (any iterable).
pub fn stream_of_streams<S, I: IntoIterator<Item = S>>(vec: I) -> impl Stream<Item = S, Error = io::Error>
  where S: Stream<Item = Bytes, Error = io::Error>
{
  stream::iter(vec.into_iter().map(|b| Ok(b)))
}

/// Generate a `ReadableByteStream<Bytes>` from a hex string.
pub fn stream_of_hex(s: &str) -> ReadableByteStream<impl Stream<Item = Bytes, Error = io::Error>> {
  ReadableByteStream::from(stream_of(Bytes::from(s.from_hex())))
}
