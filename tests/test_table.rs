extern crate bytes;
extern crate lib4bottle;

#[cfg(test)]
mod tests {
  use bytes::{Bytes};
  use lib4bottle::hex::{FromHex, ToHex};
  use lib4bottle::table::Table;

  #[test]
  fn pack() {
    let mut t = Table::new();
    t.add_bool(1);
    assert_eq!(format!("{:?}", t), "Table(B1)");
    assert_eq!(t.encode().to_hex(), "c400");
    t.add_number(10, 1000);
    assert_eq!(format!("{:?}", t), "Table(B1, N10=1000)");
    assert_eq!(t.encode().to_hex(), "c400a802e803");
    t.add_string(3, String::from("iron"));
    assert_eq!(format!("{:?}", t), "Table(B1, N10=1000, S3=\"iron\")");
    assert_eq!(t.encode().to_hex(), "c400a802e8030c0469726f6e");
  }

  #[test]
  fn unpack() {
    assert_eq!(
      format!("{:?}", Table::decode(Bytes::from("c400".from_hex())).unwrap()),
      "Table(B1)"
    );
    assert_eq!(
      format!("{:?}", Table::decode(Bytes::from("c400a802e803".from_hex())).unwrap()),
      "Table(B1, N10=1000)"
    );
    assert_eq!(
      format!("{:?}", Table::decode(Bytes::from("c400a802e8030c0469726f6e".from_hex())).unwrap()),
      "Table(B1, N10=1000, S3=\"iron\")"
    );
    assert_eq!(
      format!("{:?}", Table::decode(Bytes::from("3c0d6f6e650074776f007468726565".from_hex())).unwrap()),
      "Table(S15=\"one\\u{0}two\\u{0}three\")"
    );
  }

  #[test]
  #[should_panic(expected="Truncated header table")]
  fn unpack_truncated_1() {
    Table::decode(Bytes::from("c4".from_hex())).unwrap();
  }

  #[test]
  #[should_panic(expected="Truncated header table")]
  fn unpack_truncated_2() {
    Table::decode(Bytes::from("c401".from_hex())).unwrap();
  }

  #[test]
  #[should_panic(expected="Truncated header table")]
  fn unpack_truncated_3() {
    Table::decode(Bytes::from("c403ffff".from_hex())).unwrap();
  }
}
