//! A module for the [`PetitMap`] data structure
use crate::set::PetitSet;
use crate::CapacityError;
use core::mem::swap;

/// A map-like data structure with a fixed maximum size
///
/// This data structure does not require the [`Hash`] or [`Ord`] traits,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
///
/// Powered by a [`PetitSet`], with the values stored in a matching array of `Option<T>`.
///
/// Only `CAP` entries may be stored at once.
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
#[derive(Clone, Debug)]
pub struct PetitMap<K, V, const CAP: usize> {
    keys: PetitSet<K, CAP>,
    values: [Option<V>; CAP],
}

impl<K, V: Copy, const CAP: usize> Default for PetitMap<K, V, CAP> {
    fn default() -> Self {
        Self {
            keys: PetitSet::default(),
            values: [None; CAP],
        }
    }
}

impl<K, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Returns a reference to the value at the provided index.
    ///
    /// Returns `Some((K, V))` if the index is in-bounds and has an element.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn get_at(&self, index: usize) -> Option<(&K, &V)> {
        assert!(index <= CAP);

        self.values[index]
            .as_ref()
            .map(|value| (*self.keys.get_at(index).as_ref().unwrap(), value))
    }

    /// Returns a mutable reference to the value at the provided index.
    ///
    /// Returns `Some((&mut K, &mut V))` if the index is in-bounds and has an element
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn get_at_mut(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
        assert!(index <= CAP);

        if let Some(value) = &mut self.values[index] {
            Some((self.keys.get_at_mut(index).unwrap(), value))
        } else {
            None
        }
    }

    /// Removes the element at the provided index
    ///
    /// Returns true if an element was found
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn remove_at(&mut self, index: usize) -> bool {
        self.take_at(index).is_some()
    }

    /// Removes the key-value pair at the provided index
    ///
    /// Returns `Some((K, V))` if the index was full.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    #[must_use = "Use remove_at if the value is not needed."]
    pub fn take_at(&mut self, index: usize) -> Option<(K, V)> {
        assert!(index <= CAP);

        if let Some(key) = self.keys.take_at(index) {
            let mut removed = None;
            swap(&mut removed, &mut self.values[index]);

            // Every key must have a value:
            // if this panicked we had a malformed map
            Some((key, removed.unwrap()))
        } else {
            None
        }
    }

    /// An iterator visiting all keys in in a first-in, first-out order
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
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

impl<K: Eq, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Stores the value into the map, which can be looked up by the key
    ///
    /// Returns Ok(index) at which the key / value pair was inserted if succesful
    /// or [`Err(InsertionError::Overfull)`] if the map was already full
    pub fn try_insert(&mut self, key: K, value: V) -> Result<(usize, bool), CapacityError<(K, V)>> {
        match self.keys.try_insert(key) {
            // No duplicate, so insert a fresh value
            Ok((index, success)) => {
                self.values[index] = Some(value);
                Ok((index, success))
            }
            Err(CapacityError(key)) => Err(CapacityError((key, value))),
        }
    }

    /// Stores the value in the map, which can be looked up by the key
    ///
    /// # Panics
    /// Panics if the map was full and the key was a non-duplicate.
    pub fn insert(&mut self, key: K, value: V) {
        self.try_insert(key, value).ok().unwrap();
    }

    /// Stores the key-value pairs in the map
    ///
    /// # Panics
    /// Panics if the map was full when a non-duplicate key was inserted.
    pub fn insert_multiple(&mut self, pairs: impl IntoIterator<Item = (K, V)>) {
        for (key, value) in pairs {
            self.insert(key, value);
        }
    }

    /// Returns the index for the provided key, if it exists in the map
    pub fn find(&self, element: &K) -> Option<usize> {
        self.keys.find(element)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// Returns `Some(&V)` if the key is found
    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(index) = self.find(key) {
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
        if let Some(index) = self.find(key) {
            if let Some(ref mut value) = self.values[index] {
                return Some(value);
            }
        }
        None
    }

    /// Removes the key-value pair from the map if the key is found
    ///
    /// Returns `Some((index))` if it was found
    pub fn remove(&mut self, key: &K) -> Option<usize> {
        if let Some(index) = self.find(key) {
            // We know this is valid, because we just found the right index
            self.remove_at(index);
            Some(index)
        } else {
            None
        }
    }

    /// Removes and returns the key-value pair from the map if the key is found
    ///
    /// Returns `Some((index, (K,V)))` if it was found
    #[must_use = "Use remove if the value is not needed."]
    pub fn take(&mut self, key: &K) -> Option<(usize, (K, V))> {
        if let Some(index) = self.find(key) {
            let result = self.take_at(index).map(|pair| (index, pair));
            debug_assert!(result.is_some());
            result
        } else {
            None
        }
    }
}

impl<K: Eq, V, const CAP: usize> IntoIterator for PetitMap<K, V, CAP> {
    type Item = (K, V);
    type IntoIter = PetitMapIter<K, V, CAP>;
    fn into_iter(self) -> Self::IntoIter {
        PetitMapIter {
            map: self,
            cursor: 0,
        }
    }
}

/// An [`Iterator`] struct for [`PetitMap`]
#[derive(Clone, Debug)]
pub struct PetitMapIter<K: Eq, V, const CAP: usize> {
    map: PetitMap<K, V, CAP>,
    cursor: usize,
}

impl<K: Eq, V, const CAP: usize> Iterator for PetitMapIter<K, V, CAP> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.map.keys.next_filled_index(self.cursor) {
            self.cursor = index + 1;
            self.map.take_at(index)
        } else {
            self.cursor = CAP;
            None
        }
    }
}

impl<K: Eq, V: PartialEq, const CAP: usize> PartialEq for PetitMap<K, V, CAP> {
    fn eq(&self, other: &Self) -> bool {
        for key in self.keys() {
            if self.get(key) != other.get(key) {
                return false;
            }
        }
        true
    }
}
