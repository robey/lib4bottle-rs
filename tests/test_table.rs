extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use lib4bottle::to_hex::{FromHex, ToHex};
  use lib4bottle::table::Table;

  #[test]
  fn pack() {
    let mut m = Table::new();
    m.add_bool(1);
    assert_eq!(format!("{:?}", m), "Table(B1)");
    assert_eq!(m.encode().to_hex(), "c400");
    m.add_number(10, 1000);
    assert_eq!(format!("{:?}", m), "Table(B1, N10=1000)");
    assert_eq!(m.encode().to_hex(), "c400a802e803");
    m.add_string(3, String::from("iron"));
    assert_eq!(format!("{:?}", m), "Table(B1, N10=1000, S3=\"iron\")");
    assert_eq!(m.encode().to_hex(), "c400a802e8030c0469726f6e");
  }

  #[test]
  fn unpack() {
    assert_eq!(
      format!("{:?}", Table::decode("c400".from_hex().as_ref()).unwrap()),
      "Table(B1)"
    );
    assert_eq!(
      format!("{:?}", Table::decode("c400a802e803".from_hex().as_ref()).unwrap()),
      "Table(B1, N10=1000)"
    );
    assert_eq!(
      format!("{:?}", Table::decode("c400a802e8030c0469726f6e".from_hex().as_ref()).unwrap()),
      "Table(B1, N10=1000, S3=\"iron\")"
    );
    assert_eq!(
      format!("{:?}", Table::decode("3c0d6f6e650074776f007468726565".from_hex().as_ref()).unwrap()),
      "Table(S15=\"one\\u{0}two\\u{0}three\")"
    );
  }

  #[test]
  #[should_panic(expected="Truncated header table")]
  fn unpack_truncated_1() {
    Table::decode("c4".from_hex().as_ref()).unwrap();
  }

  #[test]
  #[should_panic(expected="Truncated header table")]
  fn unpack_truncated_2() {
    Table::decode("c401".from_hex().as_ref()).unwrap();
  }

  #[test]
  #[should_panic(expected="Truncated header table")]
  fn unpack_truncated_3() {
    Table::decode("c403ffff".from_hex().as_ref()).unwrap();
  }
}
