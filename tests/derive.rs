#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate bit_collection;

#[cfg(feature = "std")]
extern crate core;

use core::fmt::Debug;
use bit_collection::BitCollection;

macro_rules! enum_impl {
    ($(#[$attr:meta])* enum $ident:ident { $($x:ident),* $(,)* }) => {
        $(#[$attr])*
        enum $ident {
            $($x),*
        }

        impl $ident {
            fn all() -> &'static [$ident] {
                static ALL: &'static [$ident] = &[$($ident::$x),*];
                ALL
            }
        }
    }
}

enum_impl! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum Value4Enum { A, B, C, D }
}

enum_impl! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum Value16Enum {
        A, B, C, D, E, F, G, H,
        I, J, K, L, M, N, O, P,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Value4Struct(u8);

impl Value4Struct {
    fn all() -> &'static [Value4Struct] {
        static ALL: &'static [Value4Struct] = &[
            Value4Struct(0), Value4Struct(1), Value4Struct(2), Value4Struct(3)
        ];
        ALL
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Value16Struct(u8);

impl Value16Struct {
    fn all() -> &'static [Value16Struct] {
        static ALL: &'static [Value16Struct] = &[
            Value16Struct(0),  Value16Struct(1),  Value16Struct(2),  Value16Struct(3),
            Value16Struct(4),  Value16Struct(5),  Value16Struct(6),  Value16Struct(7),
            Value16Struct(8),  Value16Struct(9),  Value16Struct(10), Value16Struct(11),
            Value16Struct(12), Value16Struct(13), Value16Struct(14), Value16Struct(15),
        ];
        ALL
    }
}

fn test_collection<T: BitCollection>(all: &[T::Item])
    where
        T::Item: Copy + Eq + Debug + Into<T>
{
    for &x in all {
        let val = T::FULL;
        assert!(val.contains(x));
    }

    for (a, &b) in T::FULL.zip(all.iter()) {
        assert_eq!(a, b);
    }

    for (a, &b) in T::FULL.rev().zip(all.iter().rev()) {
        assert_eq!(a, b);
    }
}

macro_rules! impl_test {
    ($func:ident, $bit:ident, $inner:ty, #[$attr:meta]) => {
        #[test]
        fn $func() {
            #[$attr]
            #[derive(BitCollection)]
            struct Tuple($inner);

            #[$attr]
            #[derive(BitCollection)]
            struct Struct { bits: $inner }

            let all = $bit::all();

            test_collection::<Tuple>(all);
            test_collection::<Struct>(all);
        }
    }
}

impl_test!(bits4_enum,    Value4Enum,    u8,  #[bit(Value4Enum,   mask = "0b1111")]);
impl_test!(bits4_struct,  Value4Struct,  u8,  #[bit(Value4Struct, mask = "0b1111", retr = "0")]);
impl_test!(bits16_enum,   Value16Enum,   u16, #[bit(Value16Enum)]);
impl_test!(bits16_struct, Value16Struct, u16, #[bit(Value16Struct, retr = "0")]);
