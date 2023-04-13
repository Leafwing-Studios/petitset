//! A module for the [`PetitMap`] data structure

use crate::CapacityError;
use core::mem::swap;

/// A map-like data structure with a fixed maximum size
///
/// This data structure does not require the [`Hash`] or [`Ord`] traits,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
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
#[derive(Clone, Debug, Hash)]
pub struct PetitMap<K, V, const CAP: usize> {
    pub(crate) storage: [Option<(K, V)>; CAP],
}

impl<K, V, const CAP: usize> Default for PetitMap<K, V, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Create a new empty [`PetitMap`].
    ///
    /// The capacity is given by the generic parameter `CAP`.
    pub fn new() -> Self {
        PetitMap {
            storage: [(); CAP].map(|_| None),
        }
    }

    /// Returns a reference to the value at the provided index.
    ///
    /// Returns `Some((K, V))` if the index is in-bounds and has an element.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn get_at(&self, index: usize) -> Option<(&K, &V)> {
        assert!(index <= CAP);

        if let Some((key, value)) = &self.storage[index] {
            Some((key, value))
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value at the provided index.
    ///
    /// Returns `Some((&mut K, &mut V))` if the index is in-bounds and has an element
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn get_at_mut(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
        assert!(index <= CAP);

        if let Some((key, value)) = &mut self.storage[index] {
            Some((key, value))
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

        if let Some((_key, _value)) = &self.storage[index] {
            let mut removed = None;
            swap(&mut removed, &mut self.storage[index]);

            removed
        } else {
            None
        }
    }
    /// Returns an iterator over the key value pairs
    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.storage.iter().filter_map(|e| e.as_ref())
    }

    /// An iterator visiting all keys in in a first-in, first-out order
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _v)| k)
    }

    /// An iterator visiting all values in in a first-in, first-out order
    ///
    /// The item type is a `&'a V`
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_k, v)| v)
    }

    /// An iterator visiting all values in in a first-in, first-out order
    ///
    /// The item type is a `&'a mut V`
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.storage
            .iter_mut()
            .filter_map(|e| e.as_mut())
            .map(|(_k, v)| v)
    }

    /// Returns the index of the next filled slot, if any
    ///
    /// Returns None if the cursor is larger than CAP
    pub fn next_filled_index(&self, cursor: usize) -> Option<usize> {
        if cursor >= CAP {
            return None;
        }

        (cursor..CAP).find(|&i| self.storage[i].is_some())
    }

    /// Returns the index of the next empty slot, if any
    ///
    /// Returns None if the cursor is larger than CAP
    pub fn next_empty_index(&self, cursor: usize) -> Option<usize> {
        if cursor >= CAP {
            return None;
        }

        (cursor..CAP).find(|&i| self.storage[i].is_none())
    }

    /// Returns the current number of key-value pairs in the [`PetitMap`]
    pub fn len(&self) -> usize {
        self.storage.iter().filter(|e| e.is_some()).count()
    }

    /// Returns the maximum number of elements that can be stored in the [`PetitMap`]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Are there exactly 0 elements in the [`PetitMap`]?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Are there exactly CAP elements in the [`PetitMap`]?
    pub fn is_full(&self) -> bool {
        self.len() == CAP
    }

    /// Swaps the element in `index_a` with the element in `index_b`
    ///
    /// # Panics
    ///
    /// Panics if either index is greater than CAP.
    pub fn swap_at(&mut self, index_a: usize, index_b: usize) {
        assert!(index_a <= CAP);
        assert!(index_b <= CAP);

        self.storage.swap(index_a, index_b);
    }

    /// Removes all elements from the map without de-allocation
    pub fn clear(&mut self) {
        for index in 0..CAP {
            self.storage[index] = None;
        }
    }

    /// Inserts a key-value pair into the next empty index of the map,
    /// without checking for uniqueness
    ///
    /// Returns Some(index) if the operation succeeded, or None if it failed.
    ///
    /// # Warning
    /// This API is very easy to misuse and will completely break your `PetitMap` if you do.
    /// Avoid it unless you are guaranteed by construction that no duplicates exist.
    pub fn insert_unchecked(&mut self, key: K, value: V) -> Option<usize> {
        let index = self.next_empty_index(0)?;
        self.storage[index] = Some((key, value));

        Some(index)
    }
}

impl<K: Eq, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Attempts to store the value into the map, which can be looked up by the key
    ///
    /// Inserts the element if able, then returns the [`Result`] of that operation.
    /// This is either a [`SuccesfulMapInsertion`] or a [`CapacityError`].
    pub fn try_insert(
        &mut self,
        key: K,
        mut value: V,
    ) -> Result<SuccesfulMapInsertion<V>, CapacityError<(K, V)>> {
        if let Some(index) = self.find(&key) {
            let (_key, old_value) = self.get_at_mut(index).unwrap();

            // Replace the old value with the new value
            swap(&mut value, old_value);

            // Returns the old value, as the data was swapped
            Ok(SuccesfulMapInsertion::ExtantKey(value, index))
        } else if let Some(index) = self.next_empty_index(0) {
            self.storage[index] = Some((key, value));
            Ok(SuccesfulMapInsertion::NovelKey(index))
        } else {
            Err(CapacityError((key, value)))
        }
    }

    /// Stores the value in the map, which can be looked up by the key
    ///
    /// Returns a [`SuccesfulMapInsertion`], which encodes both
    /// the index at which the element is stored and whether the key was already present.
    /// If a key was already present, the previous value is also returned.
    ///
    /// # Panics
    /// Panics if the map was full and the key was a non-duplicate.
    pub fn insert(&mut self, key: K, value: V) -> SuccesfulMapInsertion<V> {
        self.try_insert(key, value)
            .expect("Inserting this key-value pair would have overflowed the map!")
    }

    /// Insert a new key-value pair at the provided index
    ///
    /// If a matching key already existed in the set, it will be moved to the supplied index.
    /// Any key-value pair that was previously there will be moved to the matching key's original index.
    ///
    /// Returns `Some((K, V))` of any element removed by this operation.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn insert_at(&mut self, key: K, value: V, index: usize) -> Option<(K, V)> {
        assert!(index <= CAP);

        if let Some(old_index) = self.find(&key) {
            self.swap_at(old_index, index);
            None
        } else if self.get_at(index).is_some() {
            let removed = self.take_at(index);
            self.storage[index] = Some((key, value));
            removed
        } else {
            self.storage[index] = Some((key, value));
            None
        }
    }

    /// Returns the index for the provided key, if it exists in the map
    pub fn find(&self, key: &K) -> Option<usize> {
        for index in 0..CAP {
            if let Some((existing_key, _val)) = &self.storage[index] {
                if *key == *existing_key {
                    return Some(index);
                }
            }
        }
        None
    }

    /// Does the map contain the provided key?
    pub fn contains_key(&self, key: &K) -> bool {
        self.find(key).is_some()
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// Returns `Some(&V)` if the key is found
    pub fn get(&self, key: &K) -> Option<&V> {
        if let Some(index) = self.find(key) {
            if let Some((_key, value)) = &self.storage[index] {
                return Some(value);
            }
        }
        None
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// Returns `Some(&K, &V)` if the key is found
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        if let Some(index) = self.find(key) {
            if let Some((key, value)) = &self.storage[index] {
                return Some((key, value));
            }
        }
        None
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    ///  Returns `Some(&mut V)` if the key is found
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(index) = self.find(key) {
            if let Some((_key, value)) = &mut self.storage[index] {
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

    /// Swaps the positions of `element_a` with the position of `element_b`
    ///
    /// Returns true if both keys were found and successfully swapped.
    pub fn swap(&mut self, key_a: &K, key_b: &K) -> bool {
        if let (Some(index_a), Some(index_b)) = (self.find(key_a), self.find(key_b)) {
            self.swap_at(index_a, index_b);
            true
        } else {
            false
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs (k, v) such that f(&k, &mut v) returns false. The elements are visited in order.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        for i in 0..self.capacity() {
            if let Some((k, v)) = self.get_at_mut(i) {
                if f(k, v) {
                    self.remove_at(i);
                }
            }
        }
    }

    /// Constructs a new [`PetitMap`] by consuming values from an iterator.
    ///
    /// The consumed values will be stored in order, with duplicate elements discarded.
    ///
    /// Returns an error if the iterator produces more than `CAP` distinct elements. The
    /// returned error will include both the element that could not be inserted, and
    /// a [`PetitMap`] containing all elements up to that point.
    ///
    /// # Example
    /// ```rust
    /// use petitset::CapacityError;
    /// use petitset::PetitMap;
    ///
    /// let elems = vec![(1, 11), (2, 21), (1, 12), (4, 41), (3, 31), (1, 13)];
    /// let set = PetitMap::<_,_, 5>::try_from_iter(elems.iter().copied());
    /// assert_eq!(set, Ok(PetitMap::from_raw_array_unchecked([Some((1,13)), Some((2, 21)), Some((4, 41)), Some((3, 31)), None])));
    ///
    /// let failed = PetitMap::<_,_, 3>::try_from_iter(elems.iter().copied());
    /// assert_eq!(failed, Err(CapacityError((PetitMap::from_raw_array_unchecked([Some((1,12)), Some((2, 21)), Some((4, 41))]), (3, 31)))));
    /// ```
    pub fn try_from_iter<I: IntoIterator<Item = (K, V)>>(
        element_iter: I,
    ) -> Result<Self, CapacityError<(Self, (K, V))>> {
        let mut map = Self::new();

        for (k, v) in element_iter {
            if let Err(CapacityError(overfull_element)) = map.try_insert(k, v) {
                return Err(CapacityError((map, overfull_element)));
            }
        }

        Ok(map)
    }

    /// Construct a [`PetitMap`] directly from an array, without checking for duplicates.
    ///
    /// It is a logic error if the keys of any two non-`None` values in the array are equal, as keys are expected to be unique.
    /// If this occurs, the [`PetitMap`] returned may behave unpredictably.
    pub fn from_raw_array_unchecked(values: [Option<(K, V)>; CAP]) -> Self {
        Self { storage: values }
    }
}

impl<K: Eq, V, const CAP: usize> Extend<(K, V)> for PetitMap<K, V, CAP> {
    /// Inserts multiple new key-value pairs to the map.
    ///
    /// Duplicate keys will overwrite existing values.
    ///
    /// # Panics
    /// Panics if the map would overflow due to the insertion of non-duplicate keys
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K: Eq, V: PartialEq, const CAP: usize> PetitMap<K, V, CAP> {
    /// Are the two [`PetitMap`]s element-for-element identical, in the same order?
    pub fn identical(&self, other: Self) -> bool {
        for i in 0..CAP {
            if self.storage[i] != other.storage[i] {
                return false;
            }
        }
        true
    }
}

impl<K: Eq, V, const CAP: usize> FromIterator<(K, V)> for PetitMap<K, V, CAP> {
    /// Panics if the iterator contains more than `CAP` distinct elements.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        PetitMap::try_from_iter(iter).unwrap()
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

impl<K: Eq, V, const CAP: usize> PetitMapIter<K, V, CAP> {
    /// Converts this iterator into the underlying [`PetitMap`]
    ///
    /// Simpler and more direct than using `.collect()`
    #[must_use]
    pub fn into_map(self) -> PetitMap<K, V, CAP> {
        self.map
    }
}

impl<K: Eq, V, const CAP: usize> Iterator for PetitMapIter<K, V, CAP> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.map.next_filled_index(self.cursor) {
            self.cursor = index + 1;
            self.map.take_at(index)
        } else {
            self.cursor = CAP;
            None
        }
    }
}

impl<K: Eq, V: PartialEq, const CAP: usize, const OTHER_CAP: usize>
    PartialEq<PetitMap<K, V, OTHER_CAP>> for PetitMap<K, V, CAP>
{
    /// Tests set-equality between the two maps
    ///
    /// This is order and cap size-independent.
    /// Use the `equivalent` method for elementwise-equality.
    ///
    /// Uses an inefficient O(n^2) algorithm due to minimal trait bounds.
    fn eq(&self, other: &PetitMap<K, V, OTHER_CAP>) -> bool {
        for key in self.keys() {
            if self.get(key) != other.get(key) {
                return false;
            }
        }
        true
    }
}

impl<K: Eq, V: Eq, const CAP: usize> Eq for PetitMap<K, V, CAP> {}

/// The `Ok` result of a successful [`PetitMap`] insertion operation
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SuccesfulMapInsertion<V> {
    /// This is a new key: the key-value pair is stored at the provided index
    NovelKey(usize),
    /// The key already existed, so the old value and the index were returned
    ExtantKey(V, usize),
}
