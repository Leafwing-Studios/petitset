extern crate alloc;

use alloc::vec::Vec;
use petitset::PetitSet;

/// Is this set sorted when iterated over?
pub fn is_sorted<T: PartialEq + Clone + Copy + Ord, const CAP: usize>(
    set: &PetitSet<T, CAP>,
) -> bool {
    let vec: Vec<T> = set.into_iter().collect();
    let mut sorted_vec = vec.clone();
    sorted_vec.sort();
    vec == sorted_vec
}
