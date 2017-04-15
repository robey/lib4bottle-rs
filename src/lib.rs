#![feature(conservative_impl_trait)]

extern crate bytes;
extern crate futures;

#[macro_use]
extern crate lazy_static;

// these could really be in a shared library somewhere:
pub mod stream_toolkit;

// intrinsic to 4bottle format:
pub mod bottle;
pub mod header;
pub mod table;
pub mod zint;
