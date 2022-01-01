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
#[derive(Clone, Debug)]
pub struct PetitMap<K, V, const CAP: usize> {
    storage: [Option<(K, V)>; CAP],
}

impl<K, V: Copy, const CAP: usize> Default for PetitMap<K, V, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Create a new empty [`PetitMap`].
    ///
    /// The capacity is given by the generic parameter `CAP`.
    pub fn new() -> Self {
        use core::mem::MaybeUninit;
        // This use of assume_init() is to get us an uninitialized array.
        // This is safe because the arrays contents are all MaybeUninit. Taken
        // from the docs for MaybeUninit.
        //
        // BLOCKED: use uninit_array() &co once they are stabilized.
        let mut data: [MaybeUninit<Option<(K, V)>>; CAP] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for element in data.iter_mut() {
            element.write(None);
        }

        PetitMap {
            storage: unsafe { data.map(|u| u.assume_init()) },
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

        for i in cursor..CAP {
            if self.storage[i].is_some() {
                return Some(i);
            }
        }
        None
    }

    /// Returns the index of the next empty slot, if any
    ///
    /// Returns None if the cursor is larger than CAP
    pub fn next_empty_index(&self, cursor: usize) -> Option<usize> {
        if cursor >= CAP {
            return None;
        }

        for i in cursor..CAP {
            if self.storage[i].is_none() {
                return Some(i);
            }
        }
        None
    }

    /// Returns the current number of key-value pairs in the [`PetitMap`]
    pub fn len(&self) -> usize {
        self.storage.iter().filter(|e| e.is_some()).count()
    }

    /// Returns the maximum number of elements that can be stored in the [`PetitMap`]
    pub fn capacity(&self) -> usize {
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

    /// Swaps the element in index_a with the element in index_b
    ///
    /// # Panics
    ///
    /// Panics if either index is greater than CAP.
    pub fn swap_at(&mut self, index_a: usize, index_b: usize) {
        assert!(index_a <= CAP);
        assert!(index_b <= CAP);

        self.storage.swap(index_a, index_b);
    }

    /// Removes all elements from the map without allocation
    pub fn clear(&mut self) {
        for index in 0..CAP {
            self.storage[index] = None;
        }
    }
}

impl<K: Eq, V, const CAP: usize> PetitMap<K, V, CAP> {
    /// Stores the value into the map, which can be looked up by the key
    ///
    /// Returns Ok(index) at which the key / value pair was inserted if succesful
    /// or [`Err(InsertionError::Overfull)`] if the map was already full
    pub fn try_insert(&mut self, key: K, value: V) -> Result<(usize, bool), CapacityError<(K, V)>> {
        if let Some(index) = self.find(&key) {
            let (_key, old_value) = self.get_at_mut(index).unwrap();
            *old_value = value;
            Ok((index, true))
        } else if let Some(index) = self.next_empty_index(0) {
            self.storage[index] = Some((key, value));
            Ok((index, true))
        } else {
            Err(CapacityError((key, value)))
        }
    }

    /// Stores the value in the map, which can be looked up by the key
    ///
    /// # Panics
    /// Panics if the map was full and the key was a non-duplicate.
    pub fn insert(&mut self, key: K, value: V) {
        self.try_insert(key, value)
            .expect("Inserting this key-value pair would have overflowed the map!");
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

    /// Insert a new element to the set at the provided index
    ///
    /// If a matching element already existed in the set, it will be moved to the supplied index.
    /// Any element that was previously there will be moved to the matching element's original index.
    ///
    /// Returns `Some(T)` of any element removed by this operation.
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
    pub fn contains(&self, key: &K) -> bool {
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

    /// Swaps the positions of element_a with the position of element_b
    ///
    /// Returns true if both keys were found and succesfully swapped.
    pub fn swap(&mut self, key_a: &K, key_b: &K) -> bool {
        if let (Some(index_a), Some(index_b)) = (self.find(key_a), self.find(key_b)) {
            self.swap_at(index_a, index_b);
            true
        } else {
            false
        }
    }

    /// Constructs a new `PetitSet` by consuming values from an iterator.
    ///
    /// The consumed values will be stored in order, with duplicate elements discarded.
    ///
    /// Returns an error if the iterator produces more than `CAP` distinct elements. The
    /// returned error will include both the element that could not be inserted, and
    /// a PetitSet containing all elements up to that point.
    ///
    /// ```
    /// use petitset::CapacityError;
    /// use petitset::PetitSet;
    ///
    /// let elems = vec![1, 2, 1, 4, 3, 1];
    /// let set = PetitSet::<_, 5>::try_from_iter(elems.iter().copied());
    /// assert_eq!(set, Ok(PetitSet::from_raw_array_unchecked([Some(1), Some(2), Some(4), Some(3), None])));
    ///
    /// let failed = PetitSet::<_, 3>::try_from_iter(elems.iter().copied());
    /// assert_eq!(failed, Err(CapacityError((PetitSet::from_raw_array_unchecked([Some(1), Some(2), Some(4)]), 3))));
    /// ```
    pub fn try_from_iter<I: IntoIterator<Item = (K, V)>>(
        element_iter: I,
    ) -> Result<Self, CapacityError<(Self, (K, V))>> {
        use core::{mem::MaybeUninit, ptr, slice};

        // This use of assume_init() is to get us an uninitialized array.
        // This is safe because the arrays contents are all MaybeUninit. Taken
        // from the docs for MaybeUninit.
        //
        // BLOCKED: use uninit_array() &co once they are stabilized.
        let mut uninit_data: [MaybeUninit<Option<(K, V)>>; CAP] =
            unsafe { MaybeUninit::uninit().assume_init() };

        // init_data will track which elements have been initialized so far, so
        // that we can scan for duplicates. This initialization is safe because
        // uninit_data is properly aligned for [Option<T>], and initialization
        // does not matter because the length is 0.
        let mut init_data: &[Option<(K, V)>] =
            unsafe { slice::from_raw_parts(uninit_data[0].as_ptr(), 0) };

        // Each iteration of this loop will initialize the element past the end of
        // init_data, then extend init_data to cover the newly-initialized element
        // and advance uninit_data by one. The invariants at each iteration are:
        //
        // - init_data contains only initialized data.
        // - init_data is a prefix of uninit_data.
        //
        // This mess is to avoid borrow checker issues checking for duplicates while
        // mutating the array.
        let mut fused_iter = element_iter.into_iter().fuse();
        while init_data.len() < uninit_data.len() {
            let index = init_data.len();

            let element = fused_iter.next();
            if let Some((key, _value)) = &element {
                for (init_key, _init_value) in init_data.iter().flatten() {
                    if *key == *init_key {
                        continue;
                    }
                }
            }

            // We are no longer using the current value of init_data, so we can safely
            // write to the next element.
            uninit_data[index].write(element);

            // This is safe because the element one past init_data was just initialized.
            init_data = unsafe { slice::from_raw_parts(uninit_data[0].as_ptr(), index + 1) }
        }

        assert_eq!(init_data.len(), CAP);
        let map = PetitMap {
            storage: unsafe {
                // This is safe because we just checked the length.
                ptr::read(init_data as *const [Option<(K, V)>] as *const [Option<(K, V)>; CAP])
            },
        };

        // Now check for any additional distinct elements in the rest of the iterator.
        for element in fused_iter {
            if !map.contains(&element.0) {
                return Err(CapacityError((map, element)));
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
