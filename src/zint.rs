use std::io;

/*
 * methods for encoding ints as:
 * - packed: LSB, with buffer length passed out-of-band
 * - length: specialized variable-length encoding that favors powers of two
 */

pub fn write_packed_int<W: io::Write>(writer: &mut W, number: u64) -> io::Result<()> {
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
  writer.write_all(&buffer[0..count])?;
  Ok(())
}

pub fn encode_packed_int(number: u64) -> Vec<u8> {
  let mut cursor = io::Cursor::new(Vec::new());
  // unwrap is ok cuz it can' really fail
  write_packed_int(&mut cursor, number).unwrap();
  cursor.into_inner()
}

pub fn read_packed_int<R: io::Read>(reader: &mut R) -> io::Result<u64> {
  let mut buffer: [u8; 1] = [ 0 ];
  let mut rv: u64 = 0;
  let mut shift: u8 = 0;

  while reader.read(&mut buffer)? > 0 && shift < 64 {
    rv += (buffer[0] as u64) << shift;
    shift += 8;
  }
  Ok(rv)
}

pub fn decode_packed_int(buffer: &[u8]) -> io::Result<u64> {
  read_packed_int(&mut io::Cursor::new(buffer))
}


/*
 * 00000000 - end of stream
 * 0xxxxxxx - 1 thru 2^7 = 128
 * 10xxxxxx - (+ 1 byte, LSB) = 2^14 = 8K
 * 110xxxxx - (+ 2 byte, LSB) = 2^21 = 2M
 * 1110xxxx - (+ 3 byte, LSB) = 2^28 = 128M
 * 1111xxxx - 2^(7+x) = any power-of-2 block size from 128 to 2^21 = 2M
 * 11111111 - end of all streams
 */
pub fn write_length<W: io::Write>(writer: &mut W, number: u32) -> io::Result<()> {
  match number {
    n if n < 128 => {
      writer.write(&[ n as u8 ])?;
      Ok(())
    }
    n if n <= (1 << 22) && (n & (n - 1) == 0) => {
      writer.write(&[ (0xf0 + log_base2(n) - 7) as u8 ])?;
      Ok(())
    }
    n if n < 8192 => {
      writer.write(&[ 0x80 + (n & 0x3f) as u8, ((n >> 6) & 0xff) as u8 ])?;
      Ok(())
    }
    n if n < (1 << 21) => {
      writer.write(&[
        0xc0 + (n & 0x1f) as u8,
        ((n >> 5) & 0xff) as u8,
        ((n >> 13) & 0xff) as u8
      ])?;
      Ok(())
    }
    n if n < (1 << 28) => {
      writer.write(&[
        0xe0 + (n & 0xf) as u8,
        ((n >> 4) & 0xff) as u8,
        ((n >> 12) & 0xff) as u8,
        ((n >> 20) & 0xff) as u8
      ])?;
      Ok(())
    }
    _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "😝"))
  }
}

pub fn encode_length(number: u32) -> Vec<u8> {
  let mut cursor = io::Cursor::new(Vec::new());
  // unwrap is ok cuz it can' really fail
  write_length(&mut cursor, number).unwrap();
  cursor.into_inner()
}

/*
 * Determine how many bytes will be needed to get the full length.
 */
pub fn length_of_length(byte: u8) -> usize {
  match byte {
    b if (b & 0xf0) == 0xf0 => 1,
    b if (b & 0x80) == 0 => 1,
    b if (b & 0xc0) == 0x80 => 2,
    b if (b & 0xe0) == 0xc0 => 3,
    b if (b & 0xf0) == 0xe0 => 4,
    _ => 0
  }
}

pub const END_OF_STREAM: u32 = 0;
pub const END_OF_ALL_STREAMS: u32 = 0xffffffff;

/*
 * Returns the length, or one of the two constants above.
 * Use `length_of_length` on the first byte to ensure that you have as many
 * bytes as you need.
 */
pub fn decode_length<R: io::Read>(reader: &mut R) -> io::Result<u32> {
  let mut buffer: [u8; 4] = [ 0; 4 ];
  reader.read_exact(&mut buffer[0..1])?;
  let total_len = length_of_length(buffer[0]);
  if total_len > 1 {
    reader.read_exact(&mut buffer[1..total_len])?;
  }

  if buffer[0] == 0xff {
    Ok(END_OF_ALL_STREAMS)
  } else if buffer[0] & 0x80 == 0 {
    Ok(buffer[0] as u32)
  } else if buffer[0] & 0xf0 == 0xf0 {
    Ok(1 << (7 + (buffer[0] & 0xf) as u32))
  } else if buffer[0] & 0xc0 == 0x80 {
    Ok(((buffer[0] & 0x3f) as u32) + ((buffer[1] as u32) << 6))
  } else if buffer[0] & 0xe0 == 0xc0 {
    Ok(((buffer[0] & 0x1f) as u32) + ((buffer[1] as u32) << 5) + ((buffer[2] as u32) << 13))
  } else {
    Ok(
      ((buffer[0] & 0xf) as u32) +
      ((buffer[1] as u32) << 4) +
      ((buffer[2] as u32) << 12) +
      ((buffer[3] as u32) << 20)
    )
  }
}

// export function readLength(stream) {
//   return stream.readPromise(1).then(prefix => {
//     if (prefix == null || prefix[0] == 0) return null;
//     if ((prefix[0] & 0x80) == 0) return prefix[0];
//     if ((prefix[0] & 0xf0) == 0xf0) return Math.pow(2, 7 + (prefix[0] & 0xf));
//     if ((prefix[0] & 0xc0) == 0x80) {
//       return stream.readPromise(1).then(data => {
//         if (data == null) return null;
//         return (prefix[0] & 0x3f) + (data[0] << 6);
//       });
//     }
//     if ((prefix[0] & 0xe0) == 0xc0) {
//       return stream.readPromise(2).then(data => {
//         if (data == null) return null;
//         return (prefix[0] & 0x3f) + (data[0] << 5) + (data[1] << 13);
//       });
//     }
//     if ((prefix[0] & 0xf0) == 0xe0) {
//       return stream.readPromise(3).then(data => {
//         if (data == null) return null;
//         return (prefix[0] & 0xf) + (data[0] << 4) + (data[1] << 12) + (data[2] << 20);
//       });
//     }
//     return null;
//   });
// }

// hacker's delight! (only works on exact powers of 2)
fn log_base2(number: u32) -> u32 {
  let mut x: u32 = number;
  x -= 1;
  x -= (x >> 1) & 0x55555555;
  x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
  x = (x + (x >> 4)) & 0x0F0F0F0F;
  x += x << 8;
  x += x << 16;
  x >>= 24;
  x
}

pub fn bytes_needed(mut number: u64) -> usize {
  let mut count = 1;//if (number & (number - 1)) == 0 { 0 } else { 1 };
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
