//! Assorted utilities for [`PetitSet`] and [`PetitMap`](crate::PetitMap), which are primarily valuable when writing tests

use crate::set::PetitSet;

impl<T: PartialEq + Clone + Copy + Ord, const CAP: usize> PetitSet<T, CAP> {
    /// Is this set sorted when iterated over?
    pub fn is_sorted(&self) -> bool {
        let vec: Vec<T> = self.into_iter().collect();
        let mut sorted_vec = vec.clone();
        sorted_vec.sort();
        vec == sorted_vec
    }
}
