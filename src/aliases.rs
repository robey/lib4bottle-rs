use bytes::{Bytes};
use futures::{Future, Stream};
use std::io;

// until we have trait aliases, found this hack at https://github.com/rust-lang/rfcs/pull/1733
pub trait ByteStream: Stream<Item = Bytes, Error = io::Error> {}
impl<T: Stream<Item = Bytes, Error = io::Error>> ByteStream for T {}

pub trait ByteStreamStream<S: ByteStream>: Stream<Item = S, Error = io::Error> {}
impl<S: ByteStream, T: Stream<Item = S, Error = io::Error>> ByteStreamStream<S> for T {}

pub trait IoFuture<A>: Future<Item = A, Error = io::Error> {}
impl<A, T: Future<Item = A, Error = io::Error>> IoFuture<A> for T {}
