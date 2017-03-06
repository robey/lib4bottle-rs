extern crate lib4bottle;

#[cfg(test)]
mod tests {
  // use std::io;
  // use std::io::Seek;
  use lib4bottle::to_hex::{FromHex, ToHex};
  use lib4bottle::bottle_header::Header;

  #[test]
  fn pack() {
    let mut m = Header::new();
    m.add_bool(1);
    assert_eq!(format!("{:?}", m), "Header(B1)");
    assert_eq!(m.encode().to_hex(), "c400");
    m.add_number(10, 1000);
    assert_eq!(format!("{:?}", m), "Header(B1, N10=1000)");
    assert_eq!(m.encode().to_hex(), "c400a802e803");
    m.add_string(3, String::from("iron"));
    assert_eq!(format!("{:?}", m), "Header(B1, N10=1000, S3=\"iron\")");
    assert_eq!(m.encode().to_hex(), "c400a802e8030c0469726f6e");
  }

  #[test]
  fn unpack() {
    assert_eq!(
      format!("{:?}", Header::decode("c400".from_hex().as_ref()).unwrap()),
      "Header(B1)"
    );
    assert_eq!(
      format!("{:?}", Header::decode("c400a802e803".from_hex().as_ref()).unwrap()),
      "Header(B1, N10=1000)"
    );
    assert_eq!(
      format!("{:?}", Header::decode("c400a802e8030c0469726f6e".from_hex().as_ref()).unwrap()),
      "Header(B1, N10=1000, S3=\"iron\")"
    );
    assert_eq!(
      format!("{:?}", Header::decode("3c0d6f6e650074776f007468726565".from_hex().as_ref()).unwrap()),
      "Header(S15=\"one\\u{0}two\\u{0}three\")"
    );
  }

  #[test]
  #[should_panic(expected="Truncated header")]
  fn unpack_truncated_1() {
    Header::decode("c4".from_hex().as_ref()).unwrap();
  }

  #[test]
  #[should_panic(expected="Truncated header")]
  fn unpack_truncated_2() {
    Header::decode("c401".from_hex().as_ref()).unwrap();
  }

  #[test]
  #[should_panic(expected="Truncated header")]
  fn unpack_truncated_3() {
    Header::decode("c403ffff".from_hex().as_ref()).unwrap();
  }
}
