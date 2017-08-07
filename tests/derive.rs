#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bit_collection;

use std::fmt::Debug;
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
        lazy_static! {
            static ref ALL: Vec<Value4Struct> = (0..4).map(|x| Value4Struct(x)).collect();
        }
        &ALL
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Value16Struct(u8);

impl Value16Struct {
    fn all() -> &'static [Value16Struct] {
        lazy_static! {
            static ref ALL: Vec<Value16Struct> = (0..16).map(|x| Value16Struct(x)).collect();
        }
        &ALL
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

macro_rules! impl_test {
    ($fn:ident, $bit:ident, $inner:ty, #[$attr:meta]) => {
        #[test]
        fn $fn() {
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
