extern crate bytes;
extern crate futures;
extern crate lib4bottle;

#[cfg(test)]
mod test_bottle {
  use bytes::{Bytes};
  use futures::{Future, Stream, stream};
  use lib4bottle::bottle::{Bottle, read_bottle, read_framed_stream, write_framed_stream};
  use lib4bottle::header::{BottleType};
  use lib4bottle::stream_toolkit::{
    ReadableByteStream, FromHex, stream_of, stream_of_hex, stream_of_streams, stream_of_vec, ToHex
  };
  use lib4bottle::table::Table;
  use std::io;

  static MAGIC_HEX: &str = "f09f8dbc0000";

  #[test]
  fn write_a_small_frame() {
    let s = write_framed_stream(stream_of(Bytes::from("010203".from_hex())));
    assert_eq!(
      s.collect().wait().unwrap().to_hex(),
      "0301020300"
    );
  }

  #[test]
  fn buffer_a_frame() {
    let s = stream_of_vec(vec![
      Bytes::from_static(b"he"),
      Bytes::from_static(b"ll"),
      Bytes::from_static(b"o sai"),
      Bytes::from_static(b"lor")
    ]);
    let b = write_framed_stream(s);
    assert_eq!(b.collect().wait().unwrap().to_hex(), "0c68656c6c6f207361696c6f7200");
  }

  #[test]
  fn write_a_small_bottle() {
    let mut t = Table::new();
    t.add_number(0, 150);
    let b = Bottle::new(BottleType::Test, t, stream::empty::<stream::Empty<Bytes, io::Error>, io::Error>());
    assert_eq!(b.encode().collect().wait().unwrap().to_hex(), format!("{}a003800196ff", MAGIC_HEX));
  }

  #[test]
  fn write_a_small_data_bottle() {
    let data = stream_of(Bytes::from("ff00ff00".from_hex()));
    let b = Bottle::new(BottleType::Test, Table::new(), stream_of_streams(vec![ data ]));
    assert_eq!(b.encode().collect().wait().unwrap().to_hex(), format!("{}a00004ff00ff0000ff", MAGIC_HEX));
  }

  #[test]
  fn write_a_nested_bottle() {
    let empty_stream = stream::empty::<stream::Empty<Bytes, io::Error>, io::Error>();
    let b1 = Bottle::new(BottleType::Test, Table::new(), empty_stream);
    let b2 = Bottle::new(BottleType::Test2, Table::new(), stream_of_streams(vec![ b1.encode() ]));
    assert_eq!(
      b2.encode().collect().wait().unwrap().to_hex(),
      format!("{}b00009{}a000ff00ff", MAGIC_HEX, MAGIC_HEX)
    );
  }

  #[test]
  fn write_a_bottle_of_several_streams() {
    let data1 = stream_of(Bytes::from("f0f0f0".from_hex()));
    let data2 = stream_of(Bytes::from("e0e0e0".from_hex()));
    let data3 = stream_of(Bytes::from("cccccc".from_hex()));
    let b = Bottle::new(BottleType::Test, Table::new(), stream_of_streams(vec![ data1, data2, data3 ]));
    assert_eq!(
      b.encode().collect().wait().unwrap().to_hex(),
      format!("{}a00003f0f0f00003e0e0e00003cccccc00ff", MAGIC_HEX)
    );
  }

  #[test]
  fn read_a_data_block() {
    let data1 = stream_of_hex("0568656c6c6f00ff");
    let (stream, future) = read_framed_stream(data1).wait().unwrap();
    assert_eq!(stream.unwrap().into_stream().collect().wait().unwrap().to_hex(), "68656c6c6f");
    assert_eq!(future.wait().unwrap().into_stream().collect().wait().unwrap().to_hex(), "ff");
  }

  #[test]
  fn read_a_continuing_data_block() {
    let data1 = ReadableByteStream::from(stream_of(Bytes::from("026865016c026c6f00ff".from_hex())));
    let (stream, future) = read_framed_stream(data1).wait().unwrap();
    assert_eq!(stream.unwrap().into_stream().collect().wait().unwrap().to_hex(), "68656c6c6f");
    assert_eq!(future.wait().unwrap().into_stream().collect().wait().unwrap().to_hex(), "ff");
  }

  #[test]
  fn read_several_streams() {
    let data1 = ReadableByteStream::from(stream_of(Bytes::from("03f0f0f00003e0e0e00003cccccc00ff".from_hex())));

    let (stream1, future1) = read_framed_stream(data1).wait().unwrap();
    assert_eq!(stream1.unwrap().into_stream().collect().wait().unwrap().to_hex(), "f0f0f0");
    let data2 = future1.wait().unwrap();

    let (stream2, future2) = read_framed_stream(data2).wait().unwrap();
    assert_eq!(stream2.unwrap().into_stream().collect().wait().unwrap().to_hex(), "e0e0e0");
    let data3 = future2.wait().unwrap();

    let (stream3, future3) = read_framed_stream(data3).wait().unwrap();
    assert_eq!(stream3.unwrap().into_stream().collect().wait().unwrap().to_hex(), "cccccc");
    let data4 = future3.wait().unwrap();

    assert_eq!(data4.into_stream().collect().wait().unwrap().to_hex(), "ff");
  }

  #[test]
  fn read_a_bottle() {
    let data1 = stream_of_hex(&format!("{}a0000363617400ff", MAGIC_HEX)[..]);
    let (bottle, end_stream) = read_bottle(data1).wait().unwrap();
    assert_eq!(bottle.header.bottle_type, BottleType::Test);
    assert_eq!(format!("{:?}", bottle.header.table), "Table()");

    let (item, stream) = bottle.streams.into_future().wait().map_err(|_| ()).unwrap();
    assert!(item.is_some());
    assert_eq!(item.unwrap().collect().wait().unwrap().to_hex(), "636174");
    let (item, _) = stream.into_future().wait().map_err(|_| ()).unwrap();
    assert!(item.is_none());

    let data2 = end_stream.wait().map_err(|_| ()).unwrap();
    assert_eq!(data2.into_stream().collect().wait().unwrap().to_hex(), "");
  }

  #[test]
  fn read_several_bottles_from_the_same_stream() {
    let data1 = stream_of_hex(&format!("{}a0000363617400ff{}b0000368617400ff", MAGIC_HEX, MAGIC_HEX)[..]);
    let (bottle, end_stream) = read_bottle(data1).wait().unwrap();
    assert_eq!(bottle.header.bottle_type, BottleType::Test);
    assert_eq!(format!("{:?}", bottle.header.table), "Table()");

    let (item, stream) = bottle.streams.into_future().wait().map_err(|_| ()).unwrap();
    assert!(item.is_some());
    assert_eq!(item.unwrap().collect().wait().unwrap().to_hex(), "636174");
    let (item, _) = stream.into_future().wait().map_err(|_| ()).unwrap();
    assert!(item.is_none());

    let data2 = end_stream.wait().map_err(|_| ()).unwrap();
    let (bottle, end_stream) = read_bottle(data2).wait().unwrap();
    assert_eq!(bottle.header.bottle_type, BottleType::Test2);
    assert_eq!(format!("{:?}", bottle.header.table), "Table()");

    let (item, stream) = bottle.streams.into_future().wait().map_err(|_| ()).unwrap();
    assert!(item.is_some());
    assert_eq!(item.unwrap().collect().wait().unwrap().to_hex(), "686174");
    let (item, _) = stream.into_future().wait().map_err(|_| ()).unwrap();
    assert!(item.is_none());

    let data3 = end_stream.wait().map_err(|_| ()).unwrap();
    assert_eq!(data3.into_stream().collect().wait().unwrap().to_hex(), "");
  }
}
