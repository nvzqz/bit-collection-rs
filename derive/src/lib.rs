#![recursion_limit="256"]

#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::{Body, Lit, MetaItem, NestedMetaItem, Ty};
use quote::Ident as Ident;

#[proc_macro_derive(BitCollection, attributes(bit))]
pub fn bit_collection(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    impl_bit_collection(&ast).parse().unwrap()
}

fn impl_bit_collection(ast: &syn::DeriveInput) -> quote::Tokens {
    let std_source: Ident = if cfg!(feature = "std") {
        "std".into()
    } else {
        "core".into()
    };
    let ops = quote! { ::#std_source::ops };

    let bit_list = ast.attrs.iter().filter_map(|a| {
        if let MetaItem::List(ref ident, ref vec) = a.value {
            if ident.as_ref() == "bit" {
                return Some(vec);
            }
        }
        None
    }).next().expect("No `bit` attribute found.");

    let item = bit_list.iter().filter_map(|x| {
        if let NestedMetaItem::MetaItem(MetaItem::Word(ref ident)) = *x {
            Some(Ident::from(ident.as_ref()))
        } else {
            None
        }
    }).next().expect("No bit item found.");

    let get_attr = |x: &str| {
        bit_list.iter().filter_map(|a| {
            if let NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident, ref val)) = *a {
                if ident == x {
                    if let Lit::Str(ref s, _) = *val {
                        return Some(Ident::from(s.as_ref()))
                    }
                }
            }
            None
        }).next()
    };

    let name = Ident::from(ast.ident.as_ref());
    let mask = get_attr("mask").unwrap_or_else(|| "!0".into());
    let backing: Ident;

    let (bits, from_x, from_x_masked) = if let Body::Struct(ref data) = ast.body {
        let field = data.fields().get(0).expect("No fields found.");

        // Extract inner type that may be surrounded by parentheses
        let extract_ty = || {
            let mut ty = &field.ty;
            loop {
                match *ty {
                    Ty::Paren(ref b) => ty = &b,
                    Ty::Path(_, ref p) => return p,
                    _ => panic!("Incompatible type: {:?}", ty),
                }
            }
        };
        backing = extract_ty()
            .segments.get(0).expect("No backing type found.")
            .ident.as_ref().into();

        let masked_x = quote!(x & #mask);

        if let Some(ref x) = field.ident {
            let bits = Ident::from(x.as_ref());
            let from = quote!(#name{#bits: x});
            let from_masked = quote!(#name{#bits: #masked_x});
            (bits, from, from_masked)
        } else {
            ("0".into(), quote!(#name(x)), quote!(#name(#masked_x)))
        }
    } else {
        panic!("Expected struct type.");
    };

    let _item_from_raw = quote!(*(&raw as *const _ as *const _));

    let item_from_raw = quote! {
        // Endian agnostic code integer to item conversion
        match ::#std_source::mem::size_of::<#item>() {
            1 => {
                let raw = raw as u8;
                #_item_from_raw
            },
            2 => {
                let raw = raw as u16;
                #_item_from_raw
            },
            4 => {
                let raw = raw as u32;
                #_item_from_raw
            },
            8 => {
                let raw = raw as u64;
                #_item_from_raw
            },
            _ => unreachable!(),
        }
    };

    let convert_x = if let Some(a) = get_attr("retr") {
        quote!(x.#a)
    } else {
        quote!(x as #backing)
    };

    quote! {
        impl From<#item> for #name {
            #[inline(always)]
            fn from(x: #item) -> #name {
                const ONE: #backing = 1;
                let x = ONE << #convert_x;
                #from_x
            }
        }

        impl From<#backing> for #name {
            #[inline(always)]
            fn from(x: #backing) -> #name {
                #from_x_masked
            }
        }

        impl #ops::Not for #name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                (!self.#bits).into()
            }
        }

        impl Iterator for #name {
            type Item = #item;

            #[inline]
            fn next(&mut self) -> Option<#item> {
                self.pop_lsb()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.len();
                (len, Some(len))
            }

            #[inline]
            fn count(self) -> usize {
                self.len()
            }

            #[inline]
            fn last(self) -> Option<#item> {
                self.msb()
            }
        }

        impl DoubleEndedIterator for #name {
            #[inline]
            fn next_back(&mut self) -> Option<#item> {
                self.pop_msb()
            }
        }

        impl ExactSizeIterator for #name {
            #[inline]
            fn len(&self) -> usize {
                self.#bits.count_ones() as _
            }
        }

        impl BitCollection for #name {
            #[inline]
            fn full() -> #name {
                let x = #mask;
                #from_x
            }

            #[inline]
            fn empty() -> #name {
                let x = 0;
                #from_x
            }

            #[inline]
            fn is_empty(&self) -> bool {
                self.#bits == 0
            }

            #[inline]
            unsafe fn lsb_unchecked(&self) -> #item {
                let raw = self.#bits.trailing_zeros();
                #item_from_raw
            }

            #[inline]
            unsafe fn msb_unchecked(&self) -> #item {
                use #std_source::mem::size_of;
                let val = size_of::<#name>() * 8 - 1;
                let raw = val ^ self.#bits.leading_zeros() as usize;
                #item_from_raw
            }

            #[inline]
            fn remove_lsb(&mut self) {
                self.#bits &= self.#bits.wrapping_sub(1);
            }

            #[inline]
            fn remove_msb(&mut self) {
                self.pop_msb();
            }

            #[inline]
            fn pop_lsb(&mut self) -> Option<Self::Item> {
                self.lsb().map(|x| {
                    self.remove_lsb();
                    x
                })
            }

            #[inline]
            fn pop_msb(&mut self) -> Option<#item> {
                self.msb().map(|x| {
                    self.#bits ^= #name::from(x).#bits;
                    x
                })
            }

            #[inline]
            fn contains<T: Into<Self>>(&self, x: T) -> bool {
                let other = x.into().#bits;
                self.#bits & other == other
            }

            #[inline]
            fn removing<T: Into<Self>>(self, x: T) -> Self {
                let x = self.#bits & !x.into().#bits;
                #from_x
            }

            #[inline]
            fn inserting<T: Into<Self>>(self, x: T) -> Self {
                let x = self.#bits | x.into().#bits;
                #from_x
            }

            #[inline]
            fn toggling<T: Into<Self>>(self, x: T) -> Self {
                let x = self.#bits ^ x.into().#bits;
                #from_x
            }

            #[inline]
            fn intersecting<T: Into<Self>>(self, x: T) -> Self {
                let x = self.#bits & x.into().#bits;
                #from_x
            }

            #[inline]
            fn remove<T: Into<Self>>(&mut self, x: T) {
                self.#bits &= !x.into().#bits;
            }

            #[inline]
            fn insert<T: Into<Self>>(&mut self, x: T) {
                self.#bits |= x.into().#bits;
            }

            #[inline]
            fn toggle<T: Into<Self>>(&mut self, x: T) {
                self.#bits ^= x.into().#bits;
            }

            #[inline]
            fn intersect<T: Into<Self>>(&mut self, x: T) {
                self.#bits &= x.into().#bits;
            }
        }
    }
}
