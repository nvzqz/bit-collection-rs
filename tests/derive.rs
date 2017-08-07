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

macro_rules! impl_test {
    ($fn:ident, $bit:ident, $inner:ty, $mask:expr) => {
        impl_test! { $fn $bit $inner; #[bit($bit, mask = $mask)] #[derive(BitCollection)] }
    };
    ($fn:ident, $bit:ident, $inner:ty) => {
        impl_test! { $fn $bit $inner; #[bit($bit)] #[derive(BitCollection)] }
    };
    ($fn:ident $bit:ident $inner:ty; $(#[$attr:meta])*) => {
        #[test]
        fn $fn() {
            $(#[$attr])*
            struct Tuple($inner);

            $(#[$attr])*
            struct Struct { bits: $inner }

            let all = $bit::all();

            test_collection::<Tuple>(all);
            test_collection::<Struct>(all);
        }
    };
}

impl_test!(bits4, Value4, u8, "0b1111");
impl_test!(bits16, Value16, u16);
