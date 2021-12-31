//! Array-backed ordered set and map data structures in Rust, optimized for stack-allocated storage of a tiny number of `Copy` elements with a fixed cap.
//!
//! This crate is designed to be used in performance-sensitive contexts with a small number of elements, where iteration is more common than look-ups and you don't mind a fixed size.
//! One particularly useful quirk is that elements are not recompacted upon removal: this can be very useful when replacing elements in a set or using the indexes that the elements are stored at in a semantic fashion.
//! Iteration order is guaranteed to be stable, on a first-in-first-out basis.

#![deny(missing_docs)]

pub mod map;
pub mod set;
pub mod utils;

pub use map::PetitMap;
pub use set::PetitSet;

/// An error returned when attempting to insert into a [`PetitSet`] or [`PetitMap`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InsertionError {
    /// The set was full before insertion was attempted
    Overfull,
    /// A matching entry already existed, and the index where it was found
    /// Cannot occur for [`PetitMap`], as existing values are overwritten
    Duplicate(usize),
}
