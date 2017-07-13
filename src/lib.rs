//! # Attributes
//!
//! - `bit_type`:
//! > The type used to represent individual bits. This attribute is required.
//!
//! - `bit_mask`:
//! > A mask indicating the valid bits of the collection.
//! > If not provided, the mask is assumed to have all bits set (i.e. `!0`).
//! > `BitCollection::full()` returns this value.
//!
//! - `bit_inner`:
//! > The suffix for retrieving the inner integer value of the bit type.
//! > It expands to `$value.$bit_inner`. Because of this, the provided retriever
//! > must be visible where the derive is located.
//! >
//! > If not provided, the bit type is assumed to be an `enum` that can be
//! > casted to an integer.
//!
//! # Examples
//!
//! In computer chess, one popular way of representing the occupants of a board
//! is through a [`Bitboard`][bitboard] type. In this type, each individual bit
//! is a square on a chess board.
//!
//! ```
//! #[macro_use]
//! extern crate bit_collection;
//! use bit_collection::BitCollection;
//!
//! #[derive(Copy, Clone)]
//! pub struct Square(u8);
//!
//! /// A set of sixty-four `Square`s.
//! #[bit_type = "Square"] // Type used to represent individual bits
//! #[bit_mask = "!0"]     // Mask representing all bits set (this is the default)
//! #[bit_retr = "0"]      // Retriever for inner value of the bit type
//! #[derive(BitCollection)]
//! pub struct Bitboard(u64);
//!
//! # fn main() {}
//! ```
//!
//! We can also represent castle rights this way.
//!
//! ```
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
//! #[bit_type = "CastleRight"]
//! #[bit_mask = "0b1111"]
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

// Reexport derive macro.
#[allow(unused_imports)]
#[macro_use]
extern crate bit_collection_derive;
#[doc(hidden)]
pub use bit_collection_derive::*;

/// A type that represents a collection of bits that can be iterated over.
pub trait BitCollection: DoubleEndedIterator + ExactSizeIterator {
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
}
