use bytes::{Buf, BufMut, Bytes, LittleEndian};
use std::io;

pub const END_OF_STREAM: u32 = 0;
pub const END_OF_ALL_STREAMS: u32 = 0xffffffff;

/// Encode a u64 as 1 - 8 bytes packed, LSB, with buffer length passed
/// out-of-band.
pub fn encode_packed_u64(number: u64) -> Bytes {
  let mut count = 0;
  let mut buffer: [u8; 8] = [ 0; 8 ];
  let mut n = number;

  while n > 255 {
    buffer[count] = (n & 0xff) as u8;
    n >>= 8;
    count += 1;
  }
  buffer[count] = (n & 0xff) as u8;
  count += 1;
  Bytes::from(&buffer[0..count])
}

/// Decode a packed u64 back into a u64.
pub fn decode_packed_u64(buffer: Bytes) -> u64 {
  let mut rv: u64 = 0;
  let mut shift: u8 = 0;
  for b in buffer.iter() {
    rv += (*b as u64) << shift;
    shift += 8;
  }
  rv
}

/// Encode a u32 as 4 bytes, LSB.
pub fn encode_u32(number: u32) -> Bytes {
  let mut vec: Vec<u8> = Vec::with_capacity(4);
  vec.put_u32::<LittleEndian>(number);
  Bytes::from(vec)
}

/// Decode a 4-byte `Bytes` back into a u32.
pub fn decode_u32(buffer: Bytes) -> u32 {
  let mut buf = io::Cursor::new(buffer);
  buf.get_u32::<LittleEndian>()
}

pub fn bytes_needed(mut number: u64) -> usize {
  let mut count = 1;
  let mut found = if (number & 0xffffffff00000000) == 0 { 0 } else { 4 };
  count += found;
  number >>= 8 * found;
  found = if (number & 0xffff0000) == 0 { 0 } else { 2 };
  count += found;
  number >>= 8 * found;
  found = if (number & 0xff00) == 0 { 0 } else { 1 };
  count += found;
  count
}
