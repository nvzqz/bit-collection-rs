#![feature(test)]

#[macro_use]
extern crate bit_collection;
extern crate test;

use test::{Bencher, black_box};
use bit_collection::BitCollection;

#[derive(Copy, Clone)]
enum U8Bit {
    _0,  _1,  _2,  _3,
    _4,  _5,  _6,  _7,
}

static ALL_BITS: [U8Bit; 8] = [
    U8Bit::_0, U8Bit::_1, U8Bit::_2, U8Bit::_3,
    U8Bit::_4, U8Bit::_5, U8Bit::_6, U8Bit::_7,
];

#[bit(U8Bit)]
#[derive(BitCollection)]
struct U8Bits(u8);

fn iter_with<T: BitCollection, U, F: FnMut(&mut T) -> Option<U>>(mut f: F) {
    let mut bits = black_box(T::full());
    while let Some(val) = f(&mut bits) {
        black_box(val);
    }
}

#[bench]
fn bench_pop_lsb(b: &mut Bencher) {
    b.iter(|| {
        iter_with(U8Bits::pop_lsb);
    });
}

#[bench]
fn bench_pop_msb(b: &mut Bencher) {
    b.iter(|| {
        iter_with(U8Bits::pop_msb);
    });
}

#[bench]
fn bench_naive_iter(b: &mut Bencher) {
    b.iter(|| {
        let mut bits = black_box(U8Bits::full());
        for &bit in &ALL_BITS {
            if bits.contains(bit) {
                bits.remove(bit);
                black_box(bit);
            }
        }
        black_box(bits);
    });
}
