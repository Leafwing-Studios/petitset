#![cfg_attr(not(feature = "thiserror_trait"), no_std)]
#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "set_algebra", allow(incomplete_features))]
#![cfg_attr(feature = "set_algebra", feature(generic_const_exprs))]

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
