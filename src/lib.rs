//! # `#[bit]` Attribute
//!
//! The `#[bit]` attribute is composed of three parts, two of which are optional
//! in some cases. The components can be provided in any order.
//!
//! ## Type:
//!
//! The type used to represent individual bits. This part is required.
//!
//! ```txt
//! #[bit(Type, ...)]
//! ```
//!
//! ## Mask:
//! A mask indicating the valid bits of the collection. This should be a
//! constant expression.
//!
//! If not provided, the mask is assumed to have all bits set (i.e. `!0`).
//!
//! `BitCollection::full()` returns this value.
//!
//! ```txt
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
//! ```txt
//! #[bit(..., retr = "inner", ...)]
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
//! use bit_collection::BitCollection;
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
//! # use bit_collection::BitCollection;
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
//! [bitboard]: https://chessprogramming.wikispaces.com/Bitboards

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std as std_provider;
#[cfg(not(feature = "std"))]
use core as std_provider;

use std_provider::iter::FromIterator;
use std_provider::ops::Not;

// Reexport derive macro.
#[allow(unused_imports)]
#[macro_use]
extern crate bit_collection_derive;
#[doc(hidden)]
pub use bit_collection_derive::*;

/// A type that represents a collection of bits that can be iterated over.
pub trait BitCollection: From<<Self as Iterator>::Item>
    + DoubleEndedIterator
    + ExactSizeIterator
    + FromIterator<<Self as Iterator>::Item>
    + Extend<<Self as Iterator>::Item>
    + Not<Output=Self>
{
    /// Returns a full instance with all bits set.
    fn full() -> Self;

    /// Returns an empty instance with no bits set.
    fn empty() -> Self;

    /// Returns whether `self` is empty.
    fn is_empty(&self) -> bool;

    /// Returns the least significant bit in `self` if `self` is not empty.
    #[inline]
    fn lsb(&self) -> Option<Self::Item> {
        if BitCollection::is_empty(self) {
            None
        } else {
            unsafe { Some(self.lsb_unchecked()) }
        }
    }

    /// Returns the most significant bit in `self` if `self` is not empty.
    #[inline]
    fn msb(&self) -> Option<Self::Item> {
        if BitCollection::is_empty(self) {
            None
        } else {
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
    fn removing<T: Into<Self>>(self, T) -> Self;

    /// Returns the result of inserting the value into `self`.
    fn inserting<T: Into<Self>>(self, T) -> Self;

    /// Returns the result of toggling the bits of the value in `self`.
    fn toggling<T: Into<Self>>(self, T) -> Self;

    /// Returns the result of intersecting the bits of the value with `self`.
    fn intersecting<T: Into<Self>>(self, T) -> Self;

    /// Returns the result of setting the bits of the value in `self` based on
    /// `condition`.
    #[inline]
    fn setting<T: Into<Self>>(self, x: T, condition: bool) -> Self {
        if condition {
            self.inserting(x)
        } else {
            self.removing(x)
        }
    }

    /// Removes the value from `self`.
    fn remove<T: Into<Self>>(&mut self, T);

    /// Inserts the value into `self`.
    fn insert<T: Into<Self>>(&mut self, T);

    /// Toggles bits of the value in `self`.
    fn toggle<T: Into<Self>>(&mut self, T);

    /// Intersects the bits of the value with `self`.
    fn intersect<T: Into<Self>>(&mut self, T);

    /// Sets the bits of the value in `self` based on `condition`.
    #[inline]
    fn set<T: Into<Self>>(&mut self, x: T, condition: bool) {
        if condition {
            self.insert(x);
        } else {
            self.remove(x);
        }
    }
}
