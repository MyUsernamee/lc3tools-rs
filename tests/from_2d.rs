use lc3tools_rs::sign_extend;
use rand::random;

#[test]
pub fn from_2d() {
    let positive = 0b1111;
    let negative = 0b10001;

    assert_eq!(sign_extend(5, positive), 0b1111);
    assert_eq!(sign_extend(5, negative), -0b1111i16 as u16);
}
