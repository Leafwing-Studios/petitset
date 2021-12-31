//! A module for the [`PetitSet`] data structure

use crate::InsertionError;
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
        self.storage.len() == 0
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

    /// Attempt to insert a new element to the set
    ///
    /// Returns Ok(index) if this succeeds, or an error if this failed due to either capacity or a duplicate entry.
    pub fn try_insert(&mut self, element: T) -> Result<usize, InsertionError> {
        if let Some(index) = self.find(&element) {
            return Err(InsertionError::Duplicate(index));
        }

        if let Some(index) = self.next_empty_index(0) {
            self.storage[index] = Some(element);
            Ok(index)
        } else {
            Err(InsertionError::Overfull)
        }
    }

    /// Insert a new element to the set
    ///
    /// # Panics
    /// Panics if the set is full and the item is not a duplicate
    pub fn insert(&mut self, element: T) {
        // Always insert
        if let Err(InsertionError::Overfull) = self.try_insert(element) {
            // But panic if the set was full
            panic!("Inserting this element would have overflowed the set!")
        }
    }

    /// Inserts multiple new elements to the set
    ///
    /// # Panics
    /// Panics if the set would overflow due to the insertion of non-duplicate items
    pub fn insert_multiple(&mut self, elements: impl IntoIterator<Item = T>) {
        for element in elements {
            self.insert(element);
        }
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

impl<T: Eq, const CAP: usize> FromIterator<T> for PetitSet<T, CAP> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set: PetitSet<T, CAP> = PetitSet::default();
        for element in iter {
            set.insert(element);
        }
        set
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
