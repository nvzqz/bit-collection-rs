//! Iterate over a collection of bits.
//!
//! # Usage
//!
//! This crate is available [on crates.io][crate] and can be used by adding the
//! following to your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bit_collection = "0.2.2"
//! ```
//!
//! and this to your crate root:
//!
//! ```
//! #[macro_use]
//! extern crate bit_collection;
//! # fn main() {}
//! ```
//!
//! # `#[bit]` Attribute
//!
//! The `#[bit]` attribute is composed of three parts, two of which are optional
//! in some cases. The components can be provided in any order.
//!
//! ## Type:
//!
//! The type used to represent individual bits. This part is required.
//!
//! ```rust,ignore
//! #[bit(Type, ...)]
//! ```
//!
//! ## Mask:
//! A mask indicating the valid bits of the collection. This should be a
//! constant expression.
//!
//! If not provided, the mask is assumed to have all bits set (i.e. `!0`).
//!
//! [`BitCollection::FULL`][FULL] returns this value.
//!
//! **Attention:** Please read the section on [safety](#safety) to ensure
//! that this is used in a _correct_ and _safe_ manner.
//!
//! ```rust,ignore
//! #[bit(..., mask = "0b11", ...)]
//! ```
//!
//! ## Retriever:
//! The suffix for retrieving the inner integer value of the bit type. It
//! expands to `$value.$retr`. Because of this, the provided retriever must be
//! visible where the derive is located.
//!
//! If not provided, the bit type is assumed to be an `enum` that can be
//! casted to an integer.
//!
//! ```rust,ignore
//! #[bit(..., retr = "inner", ...)]
//! ```
//!
//! ## Iterator:
//! The iterator for a given [`BitCollection`]. If [`BitIter`] isn't imported
//! as-is, this option allows for specifying its module path.
//!
//! ```rust,ignore
//! extern crate bit_collection as bc;
//!
//! #[bit(..., iter = "bc::BitIter", ...)]
//! ```
//!
//! # Examples
//!
//! In computer chess, one popular way of representing the occupants of a board
//! is through a [`Bitboard`][bitboard] type. In this type, each individual bit
//! is a square on a chess board.
//!
//! ```
//! # #[cfg(not(feature = "std"))]
//! # extern crate core;
//! #[macro_use]
//! extern crate bit_collection;
//! use bit_collection::*;
//!
//! #[derive(Copy, Clone)]
//! pub struct Square(u8);
//!
//! /// A set of sixty-four `Square`s.
//! #[bit(Square, mask = "!0", retr = "0")]
//! #[derive(BitCollection)]
//! pub struct Bitboard(u64);
//!
//! # fn main() {}
//! ```
//!
//! We can also represent castle rights this way.
//!
//! ```
//! # #[cfg(not(feature = "std"))]
//! # extern crate core;
//! # #[macro_use]
//! # extern crate bit_collection;
//! # use bit_collection::*;
//! #[derive(Copy, Clone)]
//! pub enum CastleRight {
//!     WhiteKingside,
//!     BlackKingside,
//!     WhiteQueenside,
//!     BlackQueenside,
//! }
//!
//! /// A set of `CastleRight`s.
//! #[bit(CastleRight, mask = "0b1111")]
//! #[derive(BitCollection)]
//! pub struct CastleRights {
//!     bits: u8
//! }
//!
//! fn iterate_over(rights: CastleRights) {
//!     for right in rights {
//!         match right {
//!             CastleRight::WhiteKingside  => { /* ... */ },
//!             CastleRight::BlackKingside  => { /* ... */ },
//!             CastleRight::WhiteQueenside => { /* ... */ },
//!             CastleRight::BlackQueenside => { /* ... */ },
//!         }
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! # Safety
//! This crate makes certain assumptions that, if unmet, may have unsafe and
//! unexpected results.
//!
//! The [`mask`](#mask) option for [`#[bit]`](#bit-attribute) _must_ have the
//! correct bits set. It _must not_ have bits set that correspond to invalid
//! instances of the bit type.
//!
//! Similarly, the bit type must be defined such that corresponding bit patterns
//! from `mask` provide legitimate values. Ask yourself, do `1 << item` and its
//! reversal (undo) operations, `pop_{lsb,msb}`, make sense in terms of the
//! provided mask?
//!
//! [crate]: https://crates.io/crates/bit_collection
//! [`BitCollection`]: trait.BitCollection.html
//! [`BitIter`]: struct.BitIter.html
//! [FULL]: trait.BitCollection.html#associatedconstant.FULL
//! [bitboard]: https://chessprogramming.wikispaces.com/Bitboards

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

use core::borrow::{Borrow, BorrowMut};
use core::iter::FromIterator;
use core::ops;

// Reexport derive macro.
#[allow(unused_imports)]
#[macro_use]
extern crate bit_collection_derive;
#[doc(hidden)]
pub use bit_collection_derive::*;

/// A type that represents a collection of bits that can be iterated over.
pub trait BitCollection: From<<Self as IntoIterator>::Item>
    + From<BitIter<Self>>
    + IntoIterator<IntoIter=BitIter<Self>>
    + FromIterator<<Self as IntoIterator>::Item>
    + Extend<<Self as IntoIterator>::Item>
    + ops::Not<Output=Self>
    + ops::BitAnd<Output=Self>
    + ops::BitAndAssign
    + ops::BitOr<Output=Self>
    + ops::BitOrAssign
    + ops::BitXor<Output=Self>
    + ops::BitXorAssign
    + ops::Sub<Output=Self>
    + ops::SubAssign
{
    /// A full instance with all bits set.
    const FULL: Self;

    /// An empty instance with no bits set.
    const EMPTY: Self;

    /// Returns the number of bits set in `self`.
    fn len(&self) -> usize;

    /// Returns whether `self` is empty.
    fn is_empty(&self) -> bool;

    /// Returns whether `self` has multiple bits set.
    fn has_multiple(&self) -> bool;

    /// Returns `self` as an iterator over itself.
    #[inline]
    fn as_iter(&mut self) -> &mut BitIter<Self> {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    /// Converts `self` into the only bit set.
    #[inline]
    fn into_bit(mut self) -> Option<Self::Item> {
        let bit = self.pop_lsb();
        if self.is_empty() { bit } else { None }
    }

    /// Returns the least significant bit in `self` if `self` is not empty.
    #[inline]
    fn lsb(&self) -> Option<Self::Item> {
        if self.is_empty() { None } else {
            unsafe { Some(self.lsb_unchecked()) }
        }
    }

    /// Returns the most significant bit in `self` if `self` is not empty.
    #[inline]
    fn msb(&self) -> Option<Self::Item> {
        if self.is_empty() { None } else {
            unsafe { Some(self.msb_unchecked()) }
        }
    }

    /// Returns the least significant bit in `self` without checking whether
    /// `self` is empty.
    unsafe fn lsb_unchecked(&self) -> Self::Item;

    /// Returns the most significant bit in `self` without checking whether
    /// `self` is empty.
    unsafe fn msb_unchecked(&self) -> Self::Item;

    /// Removes the least significant bit from `self`.
    fn remove_lsb(&mut self);

    /// Removes the most significant bit from `self`.
    fn remove_msb(&mut self);

    /// Removes the least significant bit from `self` and returns it.
    fn pop_lsb(&mut self) -> Option<Self::Item>;

    /// Removes the most significant bit from `self` and returns it.
    fn pop_msb(&mut self) -> Option<Self::Item>;

    /// Returns whether `self` contains the value.
    fn contains<T: Into<Self>>(&self, T) -> bool;

    /// Returns the result of removing the value from `self`.
    fn removing<T: Into<Self>>(self, other: T) -> Self {
        self - other.into()
    }

    /// Returns the result of inserting the value into `self`.
    fn inserting<T: Into<Self>>(self, other: T) -> Self {
        self | other.into()
    }

    /// Returns the result of toggling the bits of the value in `self`.
    fn toggling<T: Into<Self>>(self, other: T) -> Self {
        self ^ other.into()
    }

    /// Returns the result of intersecting the bits of the value with `self`.
    fn intersecting<T: Into<Self>>(self, other: T) -> Self {
        self & other.into()
    }

    /// Returns the result of setting the bits of the value in `self` based on
    /// `condition`.
    #[inline]
    fn setting<T: Into<Self>>(self, other: T, condition: bool) -> Self {
        if condition {
            self.inserting(other)
        } else {
            self.removing(other)
        }
    }

    /// Removes the value from `self`.
    #[inline]
    fn remove<T: Into<Self>>(&mut self, other: T) -> &mut Self {
        *self -= other.into();
        self
    }

    /// Inserts the value into `self`.
    #[inline]
    fn insert<T: Into<Self>>(&mut self, other: T) -> &mut Self {
        *self |= other.into();
        self
    }

    /// Toggles bits of the value in `self`.
    #[inline]
    fn toggle<T: Into<Self>>(&mut self, other: T) -> &mut Self {
        *self ^= other.into();
        self
    }

    /// Intersects the bits of the value with `self`.
    #[inline]
    fn intersect<T: Into<Self>>(&mut self, other: T) -> &mut Self {
        *self &= other.into();
        self
    }

    /// Sets the bits of the value in `self` based on `condition`.
    #[inline]
    fn set<T: Into<Self>>(&mut self, other: T, condition: bool) -> &mut Self {
        if condition {
            self.insert(other)
        } else {
            self.remove(other)
        }
    }
}

/// An iterator over the bits of a `BitCollection`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct BitIter<C: BitCollection>(pub C);

impl<C: BitCollection> From<C> for BitIter<C> {
    #[inline(always)]
    fn from(bits: C) -> Self { BitIter(bits) }
}

impl<C: BitCollection> AsRef<C> for BitIter<C> {
    #[inline(always)]
    fn as_ref(&self) -> &C { &self.0 }
}

impl<C: BitCollection> AsMut<C> for BitIter<C> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut C { &mut self.0 }
}

impl<C: BitCollection> Borrow<C> for BitIter<C> {
    #[inline(always)]
    fn borrow(&self) -> &C { self.as_ref() }
}

impl<C: BitCollection> BorrowMut<C> for BitIter<C> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut C { self.as_mut() }
}

impl<C: BitCollection> Iterator for BitIter<C> {
    type Item = C::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_lsb()
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
    fn last(self) -> Option<Self::Item> {
        self.0.msb()
    }
}

impl<C: BitCollection> DoubleEndedIterator for BitIter<C> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_msb()
    }
}

impl<C: BitCollection> ExactSizeIterator for BitIter<C> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
