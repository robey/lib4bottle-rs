#![feature(conservative_impl_trait)]

extern crate bytes;
extern crate futures;

#[macro_use]
extern crate lazy_static;

// these could really be in a shared library somewhere:
pub mod buffered_stream;
pub mod hex;
pub mod stream_helpers;
pub mod stream_reader;

// intrinsic to 4bottle format:
pub mod header;
pub mod table;
pub mod zint;
// pub mod bottle;
// pub mod compound_stream;
// pub mod bytes_stream;
// pub mod byte_stream;
