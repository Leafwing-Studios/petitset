//! Array-backed ordered set and map data structures in Rust, optimized for stack-allocated storage of a tiny number of elements with a fixed cap.
//! Your elements only need `Eq`, and this crate is`no_std` compatible!
//!
//! This crate is designed to be used in performance-sensitive contexts with a small number of elements, where iteration is more common than look-ups and you don't mind a fixed size.
//! One particularly useful quirk is that elements are not recompacted upon removal: this can be very useful when replacing elements in a set or using the indexes that the elements are stored at in a semantic fashion.
//! Iteration order is guaranteed to be stable, on a first-in-first-out basis.

#![cfg_attr(not(feature = "thiserror_trait"), no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![deny(missing_docs)]

use core::fmt::{Debug, Formatter, Result};

mod map;
pub use map::{PetitMap, SuccesfulMapInsertion};

mod set;
pub use set::{PetitSet, SuccesfulSetInsertion};

pub mod set_algebra;

#[cfg(feature = "thiserror_trait")]
use thiserror::Error;

/// An error returned when attempting to insert into a full [`PetitSet`] or [`PetitMap`].
///
/// It contains the element that could not be inserted.
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "thiserror_trait", derive(Error))]
pub struct CapacityError<T>(pub T);

impl<T> Debug for CapacityError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("A `PetitSet` or `PetitMap` has overflowed.")
            .finish()
    }
}

#[cfg(feature = "thiserror_trait")]
impl<T> std::fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self::Debug::fmt(self, f)
    }
}
