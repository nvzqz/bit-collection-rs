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
