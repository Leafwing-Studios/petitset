//! A module for the [`PetitMap`] data structure
use crate::set::PetitSet;

/// A map-like data structure with a fixed maximum size
///
/// This data structure does not require the [`Hash`] or [`Ord`] traits,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
///
/// Powered by a [`PetitSet`], with the values stored in a matching array of `Option<T>`.
///
/// Principally, this data structure should be used for relatively small maps,
/// where iteration performance, stable-order, stack-allocation and uniqueness
/// are more important than insertion or look-up speed.
/// Iteration, insertion and checking whether an element are in the map are O(CAP).
/// Retrieving a specific element is O(CAP).
/// Indexing into a particular element is O(1), as is removing an element at a specific index.
///
/// The values are stored as [`Option`]s within an array,
/// so niche optimization can significantly reduce memory footprint.
///
/// The maximum size of this type is given by the const-generic type parameter `CAP`.
/// Keys are guaranteed to be unique.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct PetitMap<K: Eq + Copy, V, const CAP: usize> {
    keys: PetitSet<K, CAP>,
    values: [Option<V>; CAP],
}

impl<K: Eq + Copy, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Returns a reference to the value corresponding to the key.
    ///
    /// Returns `Some(&V)` if the key is found
    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(index) = self.keys.find(key) {
            if let Some(ref value) = self.values[index] {
                return Some(value);
            }
        }
        None
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    ///  Returns `Some(&mut V)` if the key is found
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(index) = self.keys.find(key) {
            if let Some(ref mut value) = self.values[index] {
                return Some(value);
            }
        }
        None
    }

    /// Returns a reference to the value at the provided index.
    ///
    /// Returns `Some(&V)` if the index is in-bounds and has an element.
    pub fn get_at(&self, index: usize) -> Option<&V> {
        if let Some(reference) = &self.values[index] {
            Some(reference)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value at the provided index.
    ///
    /// Returns `Some(&mut V)` if the index is in-bounds and has an element
    pub fn get_at_mut(&mut self, index: usize) -> Option<&mut V> {
        if let Some(reference) = &mut self.values[index] {
            Some(reference)
        } else {
            None
        }
    }

    /// An iterator visiting all keys in in a first-in, first-out order
    pub fn keys(&self) -> impl Iterator<Item = K> {
        self.keys.into_iter()
    }

    /// An iterator visiting all values in in a first-in, first-out order
    ///
    /// The item type is a `&'a V`
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.values.iter().filter_map(|e| e.as_ref())
    }

    /// An iterator visiting all values in in a first-in, first-out order
    ///
    /// The item type is a `&'a mut V`
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.values.iter_mut().filter_map(|e| e.as_mut())
    }
}
