use bytes::{Bytes};

pub const END_OF_STREAM: u8 = 0;
pub const END_OF_BOTTLE: u8 = 0xff;
pub const END_OF_STREAM_ARRAY: [u8; 1] = [ END_OF_STREAM ];
pub const END_OF_BOTTLE_ARRAY: [u8; 1] = [ END_OF_BOTTLE ];

lazy_static! {
  pub static ref END_OF_STREAM_BYTES: Bytes = Bytes::from(&END_OF_STREAM_ARRAY[..]);
  pub static ref END_OF_BOTTLE_BYTES: Bytes = Bytes::from(&END_OF_BOTTLE_ARRAY[..]);
}

/// Encode a u64 as 1 - 8 bytes packed, LSB, with buffer length passed
/// out-of-band.
pub fn encode_packed_u64(number: u64) -> Bytes {
  let mut index = 0;
  let mut buffer: [u8; 8] = [ 0; 8 ];
  let mut n = number;

  while n > 255 {
    buffer[index] = (n & 0xff) as u8;
    n >>= 8;
    index += 1;
  }
  buffer[index] = (n & 0xff) as u8;
  index += 1;
  Bytes::from(&buffer[0 .. index])
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


// ----- frame length

#[derive(Debug, PartialEq)]
pub enum FrameLength {
  EndOfStream,
  EndOfBottle,
  Length(usize)
}

/// Encode a u32 length as 1 to 3 bytes, using the top 2 bits to track how
/// many additional bytes were needed.
pub fn encode_length(number: usize) -> Bytes {
  assert!(number > 0);
  assert!(number < (1 << 22));
  let mut index = 3;
  let mut buffer: [u8; 3] = [ 0; 3 ];
  let mut n = number;

  while n > 0 || (buffer[index] & 0xc0) != 0 {
    index -= 1;
    buffer[index] = (n & 0xff) as u8;
    n >>= 8;
  }
  buffer[index] = buffer[index] | ((2 - index) << 6) as u8;
  Bytes::from(&buffer[index ..])
}

/// Decode the first byte of a u32 length into a count of additional bytes,
/// and an accumulator so far.
pub fn decode_first_length_byte(byte: u8) -> (usize, FrameLength) {
  match byte {
    END_OF_STREAM => ( 0, FrameLength::EndOfStream ),
    END_OF_BOTTLE => ( 0, FrameLength::EndOfBottle ),
    _ => ( ((byte & 0xc0) >> 6) as usize, FrameLength::Length((byte & 0x3f) as usize) )
  }
}

/// Decode any remaining bytes from a length encoding.
pub fn decode_length(length: FrameLength, bytes: &[u8]) -> FrameLength {
  match length {
    FrameLength::EndOfStream => length,
    FrameLength::EndOfBottle => length,
    FrameLength::Length(accumulator) => {
      let mut n: usize = accumulator;
      for b in bytes.iter() {
        n = (n << 8) | (*b as usize);
      }
      FrameLength::Length(n as usize)
    }
  }
}
