#![recursion_limit="512"]

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
    let std: Ident = if cfg!(feature = "std") {
        "std".into()
    } else {
        "core".into()
    };

    let bit_list = ast.attrs.iter().filter_map(|a| {
        if let MetaItem::List(ref ident, ref vec) = a.value {
            if ident.as_ref() == "bit" {
                return Some(vec);
            }
        }
        None
    }).next().expect("No `#[bit]` attribute found.");

    let item = bit_list.iter().filter_map(|x| {
        if let NestedMetaItem::MetaItem(MetaItem::Word(ref ident)) = *x {
            Some(Ident::from(ident.as_ref()))
        } else {
            None
        }
    }).next().expect("No bit item found: `#[bit(Item)]`.");

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

    let zero = ::syn::Ident::new("0");
    let name = Ident::from(ast.ident.as_ref());
    let mask = get_attr("mask").unwrap_or_else(|| "!0".into());
    let iter = get_attr("iter").unwrap_or_else(|| "BitIter".into());
    let backing: Ident;

    let (bits, from_x, from_x_masked, full, empty) = if let Body::Struct(ref data) = ast.body {
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

        if let Some(ref bits) = field.ident {
            let from        = quote!(#name{#bits: x});
            let from_masked = quote!(#name{#bits: #masked_x});
            let full        = quote!(#name{#bits: #mask});
            let empty       = quote!(#name{#bits: 0});
            (bits, from, from_masked, full, empty)
        } else {
            (
                &zero,
                quote!(#name(x)),
                quote!(#name(#masked_x)),
                quote!(#name(#mask)),
                quote!(#name(0))
            )
        }
    } else {
        panic!("Expected struct type.");
    };

    let item_from_raw = quote! {
        // Endian agnostic code integer to item conversion
        use #std::mem::transmute_copy;
        match ::#std::mem::size_of::<#item>() {
            1 => transmute_copy(&(raw as u8)),
            2 => transmute_copy(&(raw as u16)),
            4 => transmute_copy(&(raw as u32)),
            8 => transmute_copy(&(raw as u64)),
            _ => unreachable!(),
        }
    };

    let convert_x = if let Some(retr) = get_attr("retr") {
        quote!(x.#retr)
    } else {
        quote!(x as #backing)
    };

    quote! {
        impl From<#item> for #name {
            #[inline(always)]
            fn from(item: #item) -> #name {
                const ONE: #backing = 1;
                let x = item;
                let x = ONE << #convert_x;
                #from_x
            }
        }

        impl From<#iter<#name>> for #name {
            #[inline(always)]
            fn from(iter: #iter<#name>) -> #name {
                iter.0
            }
        }

        impl From<#backing> for #name {
            #[inline(always)]
            fn from(inner: #backing) -> #name {
                let x = inner;
                #from_x_masked
            }
        }

        impl<'a, T: Clone + Into<#name>> From<&'a T> for #name {
            #[inline]
            fn from(r: &'a T) -> #name {
                r.clone().into()
            }
        }

        impl<'a, T: Clone + Into<#name>> From<&'a mut T> for #name {
            #[inline]
            fn from(r: &'a mut T) -> #name {
                r.clone().into()
            }
        }

        impl<T: Into<#name>> ::#std::ops::BitAnd<T> for #name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: T) -> Self {
                let x = self.#bits.bitand(rhs.into().#bits);
                #from_x
            }
        }

        impl<T: Into<#name>> ::#std::ops::BitAndAssign<T> for #name {
            #[inline]
            fn bitand_assign(&mut self, rhs: T) {
                self.#bits.bitand_assign(rhs.into().#bits);
            }
        }

        impl<T: Into<#name>> ::#std::ops::BitOr<T> for #name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: T) -> Self {
                let x = self.#bits.bitor(rhs.into().#bits);
                #from_x
            }
        }

        impl<T: Into<#name>> ::#std::ops::BitOrAssign<T> for #name {
            #[inline]
            fn bitor_assign(&mut self, rhs: T) {
                self.#bits.bitor_assign(rhs.into().#bits);
            }
        }

        impl<T: Into<#name>> ::#std::ops::BitXor<T> for #name {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: T) -> Self {
                let x = self.#bits.bitxor(rhs.into().#bits);
                #from_x
            }
        }

        impl<T: Into<#name>> ::#std::ops::BitXorAssign<T> for #name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: T) {
                self.#bits.bitxor_assign(rhs.into().#bits);
            }
        }

        impl<T: Into<#name>> ::#std::ops::Sub<T> for #name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: T) -> Self {
                let x = self.#bits & !rhs.into().#bits;
                #from_x
            }
        }

        impl<T: Into<#name>> ::#std::ops::SubAssign<T> for #name {
            #[inline]
            fn sub_assign(&mut self, rhs: T) {
                self.#bits &= !rhs.into().#bits;
            }
        }

        impl ::#std::ops::Not for #name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                (!self.#bits).into()
            }
        }

        impl<T: Into<#name>> ::#std::iter::FromIterator<T> for #name {
            #[inline]
            fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
                iter.into_iter().fold(Self::EMPTY, BitCollection::inserting)
            }
        }

        impl<T: Into<#name>> Extend<T> for #name {
            #[inline]
            fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
                use #std::iter::FromIterator;
                self.insert(Self::from_iter(iter));
            }
        }

        impl IntoIterator for #name {
            type IntoIter = #iter<Self>;
            type Item = #item;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                self.into()
            }
        }

        impl BitCollection for #name {
            const FULL: Self = #full;

            const EMPTY: Self = #empty;

            #[inline]
            fn len(&self) -> usize {
                self.#bits.count_ones() as _
            }

            #[inline]
            fn is_empty(&self) -> bool {
                self.#bits == 0
            }

            #[inline]
            fn has_multiple(&self) -> bool {
                self.#bits & self.#bits.wrapping_sub(1) != 0
            }

            #[inline]
            unsafe fn lsb_unchecked(&self) -> #item {
                let raw = self.#bits.trailing_zeros();
                #item_from_raw
            }

            #[inline]
            unsafe fn msb_unchecked(&self) -> #item {
                use #std::mem::size_of;
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
            fn contains<T: Into<Self>>(&self, other: T) -> bool {
                let other = other.into().#bits;
                self.#bits & other == other
            }
        }
    }
}
