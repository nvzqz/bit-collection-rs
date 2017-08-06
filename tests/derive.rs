#[macro_use]
extern crate bit_collection;

use std::fmt::Debug;
use bit_collection::BitCollection;

macro_rules! count {
    () => { 0 };
    ($x:tt) => { 1 };
    ($x:tt $y:tt $z:tt $w:tt $($rest:tt)*) => { 4 + count!($($rest)*) };
}

macro_rules! enum_impl {
    ($(#[$attr:meta])* enum $ident:ident { $($x:ident),* $(,)* }) => {
        $(#[$attr])*
        enum $ident {
            $($x),*
        }

        impl $ident {
            fn all() -> [$ident; count!($($x)*)] { [$($ident::$x),*] }
        }
    }
}

enum_impl! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum Value4 { A, B, C, D }
}

enum_impl! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum Value16 {
        A, B, C, D, E, F, G, H,
        I, J, K, L, M, N, O, P,
    }
}

fn test_collection<T: BitCollection>(all: &[T::Item])
    where
        T::Item: Copy + Eq + Debug + Into<T>
{
    for &x in all {
        let val = T::full();
        assert!(val.contains(x));
    }

    assert_eq!(T::full().collect::<Vec<_>>(), all);

    let rev_a = T::full().rev().collect::<Vec<_>>();
    let rev_b = all.iter().rev().map(|&x| x).collect::<Vec<_>>();
    assert_eq!(&rev_a, &rev_b);
}

#[test]
fn bits4_tuple_iter() {
    #[bit_mask = "0b1111"]
    #[bit_type = "Value4"]
    #[derive(BitCollection)]
    struct Bits4Tuple(u8);

    test_collection::<Bits4Tuple>(&Value4::all());
}

#[test]
fn bits4_struct_iter() {
    #[bit_mask = "0b1111"]
    #[bit_type = "Value4"]
    #[derive(BitCollection)]
    struct Bits4Struct { bits: u8 }

    test_collection::<Bits4Struct>(&Value4::all());
}

#[test]
fn bits16_tuple_iter() {
    // Should work regardless of parentheses
    #[bit_type = "Value16"]
    #[derive(BitCollection)]
    struct Bits16Tuple((((((((u16))))))));

    test_collection::<Bits16Tuple>(&Value16::all());
}

#[test]
fn bits16_struct_iter() {
    #[bit_type = "Value16"]
    #[derive(BitCollection)]
    struct Bits16Struct {
        // Should work with any identifier
        inner: u16
    }

    test_collection::<Bits16Struct>(&Value16::all());
}
