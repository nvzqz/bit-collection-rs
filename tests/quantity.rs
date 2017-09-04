#[macro_use]
extern crate bit_collection;
use bit_collection::*;

#[derive(Copy, Clone)]
struct U8Bit(u8);

#[derive(BitCollection)]
#[bit(U8Bit, retr = "0")]
struct U8Bits(u8);

#[test]
fn quantity() {
    for b in (0..0xFF).map(|x| U8Bits(x)) {
        let q = match b.len() {
            0 => Quantity::None,
            1 => Quantity::Single,
            _ => Quantity::Multiple,
        };
        assert_eq!(q, b.quantity());
    }
}
