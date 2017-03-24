use bytes::{Bytes};
use std::fmt;
use std::io;
use futures::{Future, future, Stream};

use stream_helpers::{stream_of_vec};
use stream_reader::{StreamReader};
use table::{Table};

static MAGIC: [u8; 4] = [ 0xf0, 0x9f, 0x8d, 0xbc ];
const VERSION: u8 = 0;

const MAX_TABLE_SIZE: usize = 4095;

/// Bottle type (0 - 15) as defined in the spec.
#[derive(Clone, Debug)]
pub enum BottleType {
  File = 0,
  Hashed = 1,
  Encrypted = 3,
  Compressed = 4,
  // for tests:
  Test = 10,
  Test2 = 11
}

fn decode_bottle_type(btype: u8) -> Result<BottleType, io::Error> {
  match btype {
    0 => Ok(BottleType::File),
    1 => Ok(BottleType::Hashed),
    3 => Ok(BottleType::Encrypted),
    4 => Ok(BottleType::Compressed),
    10 => Ok(BottleType::Test),
    11 => Ok(BottleType::Test2),
    _ => Err(unknown_bottle_type_error(btype))
  }
}

/// The header (magic bytes, bottle type, and key/value table) for a bottle.
pub struct Header {
  bottle_type: BottleType,
  table: Table
}

impl Header {
  pub fn new(bottle_type: BottleType, table: Table) -> Header {
    Header { bottle_type: bottle_type, table: table }
  }

  /// Generate a stream of the serialized format of this header.
  pub fn encode(&self) -> impl Stream<Item = Bytes, Error = io::Error> {
    let table_bytes = self.table.encode();
    let bottle_type_u8 = self.bottle_type.clone() as u8;
    assert!(table_bytes.len() <= MAX_TABLE_SIZE);
    let version: [u8; 4] = [
      VERSION,
      0,
      (bottle_type_u8 << 4) | ((table_bytes.len() >> 8) & 0xf) as u8,
      (table_bytes.len() & 0xff) as u8
    ];
    stream_of_vec(vec![ Bytes::from_static(&MAGIC), Bytes::from(&version[..]), Bytes::from(table_bytes) ])
  }

  /// Read a bottle header from a `Stream<Bytes>`, and return the header and
  /// the remainder of the stream.
  pub fn decode<S>(s: S)
    -> impl Future<Item = (Header, impl Stream<Item = Bytes, Error = io::Error>), Error = io::Error>
    where S: Stream<Item = Bytes, Error = io::Error>
  {
    StreamReader::read_exact(s, 8).and_then(|( frame, s )| {
      future::result(check_magic(frame.pack())).and_then(|( bottle_type, header_length )| {
        println!("type {:?}, header len {}", bottle_type, header_length);
        StreamReader::read_exact(s, header_length).and_then(|( frame, s )| {
          future::result(Table::decode(frame.pack())).map(|header| {
            ( Header::new(bottle_type, header), s )
          })
        })
      })
    })
  }
}

impl fmt::Debug for Header {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Header({:?}, {:?})", self.bottle_type, self.table)
  }
}

fn check_magic(buffer: Bytes) -> Result<(BottleType, usize), io::Error> {
  if buffer.slice(0, 4) != &MAGIC[..] {
    return Err(bad_magic_error());
  }
  if buffer[4] != VERSION || buffer[5] != 0 {
    return Err(bad_version_error(buffer[4], buffer[5]));
  }
  let btype = decode_bottle_type((buffer[6] >> 4) & 0xf)?;
  let header_length = (((buffer[6] & 0xf) as usize) << 8) + (buffer[7] as usize);
  Ok((btype, header_length))
}

fn bad_magic_error() -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, "Incorrect magic (not a 4bottle archive)")
}

fn bad_version_error(version: u8, extra: u8) -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, format!("Incompatible version: {}, {}", version, extra))
}

fn unknown_bottle_type_error(bottle_type: u8) -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, format!("Unknown bottle type: {}", bottle_type))
}
