//! A module for the [`PetitSet`] data structure

use crate::CapacityError;
use core::mem::swap;

/// A set-like data structure with a fixed maximum size
///
/// This data structure does not require the [`Hash`] or [`Ord`] traits,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
///
/// Principally, this data structure should be used for relatively small sets,
/// where iteration performance, stable-order, stack-allocation and uniqueness
/// are more important than insertion or look-up speed.
/// Iteration, insertion and checking whether an element is in the set are O(CAP).
/// Indexing into a particular element is O(1), as is removing an element at a specific index.
///
/// The values are stored as [`Option`]s within an array,
/// so niche optimization can significantly reduce memory footprint.
///
/// The maximum size of this type is given by the const-generic type parameter `CAP`.
/// Entries in this structure are guaranteed to be unique.
#[derive(Debug, Clone, Eq)]
pub struct PetitSet<T: Eq, const CAP: usize> {
    storage: [Option<T>; CAP],
}

impl<T: Eq, const CAP: usize> Default for PetitSet<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq, const CAP: usize> PetitSet<T, CAP> {
    /// Create a new empty [`PetitSet`].
    ///
    /// The capacity is given by the generic parameter `CAP`.
    pub fn new() -> Self {
        use core::mem::MaybeUninit;
        // This use of assume_init() is to get us an uninitialized array.
        // This is safe because the arrays contents are all MaybeUninit. Taken
        // from the docs for MaybeUninit.
        //
        // BLOCKED: use uninit_array() &co once they are stabilized.
        let mut data: [MaybeUninit<Option<T>>; CAP] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for element in data.iter_mut() {
            element.write(None);
        }

        PetitSet {
            storage: unsafe { data.map(|u| u.assume_init()) },
        }
    }

    /// Returns the current number of elements in the [`PetitSet`]
    pub fn len(&self) -> usize {
        self.storage.iter().filter(|e| e.is_some()).count()
    }

    /// Returns an iterator over the elements of the [`PetitSet`]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.storage.iter().filter_map(|e| e.as_ref())
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

    /// Return the capacity of the [`PetitSet`]
    #[must_use]
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Are there exactly 0 elements in the [`PetitSet`]?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the provided index of the underlying array
    ///
    /// Returns `Some(&T)` if the index is in-bounds and has an element
    #[must_use]
    pub fn get_at(&self, index: usize) -> Option<&T> {
        if let Some(reference) = &self.storage[index] {
            Some(reference)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the provided index of the underlying array
    ///
    /// Returns `Some(&mut T)` if the index is in-bounds and has an element
    #[must_use]
    pub fn get_at_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(reference) = &mut self.storage[index] {
            Some(reference)
        } else {
            None
        }
    }

    /// Returns the index for the provided element, if it exists in the set
    pub fn find(&self, element: &T) -> Option<usize> {
        for index in 0..CAP {
            if let Some(existing_element) = &self.storage[index] {
                if *element == *existing_element {
                    return Some(index);
                }
            }
        }
        None
    }

    /// Is the provided element in the set?
    #[must_use]
    pub fn contains(&self, element: &T) -> bool {
        self.find(element).is_some()
    }

    /// Attempt to insert a new element to the set in the first available slot.
    ///
    /// Returns the index of the element along with either true if the value was or false if it was already present.
    ///
    /// Returns a `CapacityError` if the element is not already present and the set is full.
    pub fn try_insert(&mut self, element: T) -> Result<(usize, bool), CapacityError<T>> {
        if let Some(index) = self.find(&element) {
            return Ok((index, false));
        }

        if let Some(index) = self.next_empty_index(0) {
            self.storage[index] = Some(element);
            Ok((index, true))
        } else {
            Err(CapacityError(element))
        }
    }

    /// Insert a new element to the set in the first available slot
    ///
    /// Returns the index of the element along with either true if the value was or false if it was already present.
    ///
    /// # Panics
    /// Panics if the set is full and the item is not a duplicate
    pub fn insert(&mut self, element: T) -> (usize, bool) {
        self.try_insert(element)
            .ok()
            .expect("Inserting this element would have overflowed the set!")
    }

    /// Inserts multiple new elements to the set. Duplicate elements are discarded.
    ///
    /// # Panics
    /// Panics if the set would overflow due to the insertion of non-duplicate items
    pub fn extend(&mut self, elements: impl IntoIterator<Item = T>) {
        for element in elements {
            self.insert(element);
        }
    }

    /// Inserts multiple new elements to the set. Duplicate elements are discarded.
    ///
    /// Returns a `CapacityError` if the extension cannot be completed because the set is full.
    pub fn try_extend(
        &mut self,
        elements: impl IntoIterator<Item = T>,
    ) -> Result<(), CapacityError<T>> {
        for element in elements {
            self.try_insert(element)?;
        }
        Ok(())
    }

    /// Removes all elements from the set without allocation
    pub fn clear(&mut self) {
        for element in self.storage.iter_mut() {
            *element = None;
        }
    }

    /// Removes the element from the set, if it exists
    ///
    /// Returns `Some(index)` if the element was found, or `None` if no matching element is found
    pub fn remove(&mut self, element: &T) -> Option<usize> {
        if let Some(index) = self.find(element) {
            self.storage[index] = None;
            Some(index)
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

    /// Removes an element from the set, if it exists, returning
    /// both the value that compared equal and the index at which
    /// it was stored.
    #[must_use = "Use remove if the value is not needed."]
    pub fn take(&mut self, element: &T) -> Option<(usize, T)> {
        if let Some(index) = self.find(element) {
            self.take_at(index).map(|removed| (index, removed))
        } else {
            None
        }
    }

    /// Removes the element at the provided index
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    #[must_use = "Use remove_at if the value is not needed."]
    pub fn take_at(&mut self, index: usize) -> Option<T> {
        assert!(index <= CAP);

        let mut removed = None;
        swap(&mut removed, &mut self.storage[index]);
        removed
    }

    /// Insert a new element to the set at the provided index
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    /// If a matching element already exists in the set, `None` will be returned.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    pub fn insert_at(&mut self, element: T, index: usize) -> Option<T> {
        assert!(index <= CAP);

        if self.contains(&element) {
            return None;
        }

        let mut element = Some(element);
        swap(&mut element, &mut self.storage[index]);
        element
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
    pub fn try_from_iter<I: IntoIterator<Item = T>>(
        value_iter: I,
    ) -> Result<Self, CapacityError<(Self, T)>> {
        use core::{mem::MaybeUninit, ptr, slice};

        // This use of assume_init() is to get us an uninitialized array.
        // This is safe because the arrays contents are all MaybeUninit. Taken
        // from the docs for MaybeUninit.
        //
        // BLOCKED: use uninit_array() &co once they are stabilized.
        let mut uninit_data: [MaybeUninit<Option<T>>; CAP] =
            unsafe { MaybeUninit::uninit().assume_init() };

        // init_data will track which elements have been initialized so far, so
        // that we can scan for duplicates. This initialization is safe because
        // uninit_data is properly aligned for [Option<T>], and initialization
        // does not matter because the length is 0.
        let mut init_data: &[Option<T>] =
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
        let mut value_iter = value_iter.into_iter().fuse();
        while init_data.len() < uninit_data.len() {
            let index = init_data.len();

            let value = value_iter.next();
            if value.is_some() && init_data.contains(&value) {
                continue;
            }

            // We are no longer using the current value of init_data, so we can safely
            // write to the next element.
            uninit_data[index].write(value);

            // This is safe because the element one past init_data was just initialized.
            init_data = unsafe { slice::from_raw_parts(uninit_data[0].as_ptr(), index + 1) }
        }

        assert_eq!(init_data.len(), CAP);
        let set = PetitSet {
            storage: unsafe {
                // This is safe because we just checked the length.
                ptr::read(init_data as *const [Option<T>] as *const [Option<T>; CAP])
            },
        };

        // Now check for any additional distinct elements in the rest of the iterator.
        for value in value_iter {
            if !set.contains(&value) {
                return Err(CapacityError((set, value)));
            }
        }
        Ok(set)
    }

    /// Construct a PetitSet directly from an array, without checking for duplicates.
    ///
    /// It is a logic error if any two non-`None` values in the array compare equal. If this occurs, the `PetitSet` returned may behave unpredictably.
    pub fn from_raw_array_unchecked(values: [Option<T>; CAP]) -> Self {
        Self { storage: values }
    }
}

impl<T: Eq, const CAP: usize> FromIterator<T> for PetitSet<T, CAP> {
    /// Panics if the iterator contains more than `CAP` distinct elements.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set: PetitSet<T, CAP> = PetitSet::default();
        for element in iter {
            set.insert(element);
        }
        set
    }
}

impl<T: Eq, const CAP: usize> IntoIterator for PetitSet<T, CAP> {
    type Item = T;
    type IntoIter = PetitSetIter<T, CAP>;
    fn into_iter(self) -> Self::IntoIter {
        PetitSetIter {
            set: self,
            cursor: 0,
        }
    }
}

/// An [`Iterator`] struct for [`PetitSet`]
#[derive(Clone, Debug)]
pub struct PetitSetIter<T: Eq, const CAP: usize> {
    set: PetitSet<T, CAP>,
    cursor: usize,
}

impl<T: Eq, const CAP: usize> Iterator for PetitSetIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.set.next_filled_index(self.cursor) {
            self.cursor = index + 1;
            let result = self.set.take_at(index);
            debug_assert!(result.is_some());
            result
        } else {
            self.cursor = CAP;
            None
        }
    }
}

impl<T: Eq, const CAP: usize> PartialEq for PetitSet<T, CAP> {
    /// Uses an inefficient O(n^2) approach to avoid introducing additional trait bounds
    fn eq(&self, other: &Self) -> bool {
        // Two sets cannot be equal if their cardinality differs
        if self.len() != other.len() {
            return false;
        }

        for item in self.iter() {
            let mut match_found = false;
            for other_item in other.iter() {
                // If a match can be found, we do not need to find another match for `item`
                if item == other_item {
                    match_found = true;
                    break;
                }
            }
            // If no match can be found, the sets cannot match
            if !match_found {
                return false;
            }
        }
        // Matches must be found for all items in the set for the them to be equal
        true
    }
}
