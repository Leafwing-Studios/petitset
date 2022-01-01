//! A module for the [`PetitSet`] data structure

use crate::PetitMap;
use crate::{map::SuccesfulMapInsertion, CapacityError};

/// A set-like data structure with a fixed maximum size
///
/// This data structure does not require the [`Hash`] or [`Ord`] traits,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
///
/// Under the hood, this is a [`PetitMap<T, (), CAP>`].
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
#[derive(Debug, Clone)]
pub struct PetitSet<T, const CAP: usize> {
    map: PetitMap<T, (), CAP>,
}

impl<T, const CAP: usize> Default for PetitSet<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> PetitSet<T, CAP> {
    /// Create a new empty [`PetitSet`].
    ///
    /// The capacity is given by the generic parameter `CAP`.
    pub fn new() -> Self {
        Self {
            map: PetitMap::new(),
        }
    }

    /// Returns the index of the next filled slot, if any
    ///
    /// Returns None if the cursor is larger than CAP
    pub fn next_filled_index(&self, cursor: usize) -> Option<usize> {
        self.map.next_filled_index(cursor)
    }

    /// Returns the index of the next empty slot, if any
    ///
    /// Returns None if the cursor is larger than CAP
    pub fn next_empty_index(&self, cursor: usize) -> Option<usize> {
        self.map.next_empty_index(cursor)
    }

    /// Return the capacity of the [`PetitSet`]
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the current number of elements in the [`PetitSet`]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Are there exactly 0 elements in the [`PetitSet`]?
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Are there exactly CAP elements in the [`PetitSet`]?
    pub fn is_full(&self) -> bool {
        self.map.is_full()
    }

    /// Returns an iterator over the elements of the [`PetitSet`]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.iter().map(|(k, _v)| k)
    }

    /// Returns a reference to the provided index of the underlying array
    ///
    /// Returns `Some(&T)` if the index is in-bounds and has an element
    pub fn get_at(&self, index: usize) -> Option<&T> {
        self.map.get_at(index).map(|(k, _v)| k)
    }

    /// Returns a mutable reference to the provided index of the underlying array
    ///
    /// Returns `Some(&mut T)` if the index is in-bounds and has an element
    pub fn get_at_mut(&mut self, index: usize) -> Option<&mut T> {
        self.map.get_at_mut(index).map(|(k, _v)| k)
    }

    /// Removes all elements from the set without allocation
    pub fn clear(&mut self) {
        self.map.clear()
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

    /// Removes the element at the provided index
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    ///
    /// # Panics
    /// Panics if the provided index is larger than CAP.
    #[must_use = "Use remove_at if the value is not needed."]
    pub fn take_at(&mut self, index: usize) -> Option<T> {
        self.map.take_at(index).map(|(k, _v)| k)
    }

    /// Swaps the element in index_a with the element in index_b
    ///
    /// # Panics
    ///
    /// Panics if either index is greater than CAP.
    pub fn swap_at(&mut self, index_a: usize, index_b: usize) {
        self.map.swap_at(index_a, index_b);
    }
}

impl<T: Eq, const CAP: usize> PetitSet<T, CAP> {
    /// Returns the index for the provided element, if it exists in the set
    pub fn find(&self, element: &T) -> Option<usize> {
        self.map.find(element)
    }

    /// Is the provided element in the set?
    #[must_use]
    pub fn contains(&self, element: &T) -> bool {
        self.find(element).is_some()
    }

    /// Attempt to insert a new element to the set in the first available slot.
    ///
    /// Inserts the element if able, then returns the [`Result`] of that operation.
    /// This is either a [`SuccesfulSetInsertion`] or a [`CapacityError`].
    pub fn try_insert(&mut self, element: T) -> Result<SuccesfulSetInsertion, CapacityError<T>> {
        match self.map.try_insert(element, ()) {
            Ok(sucess) => match sucess {
                SuccesfulMapInsertion::NovelKey(index) => {
                    Ok(SuccesfulSetInsertion::NovelElenent(index))
                }
                SuccesfulMapInsertion::ExtantKey(_val, index) => {
                    Ok(SuccesfulSetInsertion::ExtantElement(index))
                }
            },
            Err(CapacityError((key, _value))) => Err(CapacityError(key)),
        }
    }

    /// Insert a new element to the set in the first available slot
    ///
    /// Returns a [`SuccesfulSetInsertion`], which encodes both the index at which the element is stored
    /// and whether the element was already present.
    ///
    /// # Panics
    /// Panics if the set is full and the item is not a duplicate
    pub fn insert(&mut self, element: T) -> SuccesfulSetInsertion {
        self.try_insert(element)
            .expect("Inserting this element would have overflowed the set!")
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
    pub fn insert_at(&mut self, element: T, index: usize) -> Option<T> {
        self.map.insert_at(element, (), index).map(|(k, _v)| k)
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

    /// Removes the element from the set, if it exists
    ///
    /// Returns `Some(index)` if the element was found, or `None` if no matching element is found
    pub fn remove(&mut self, element: &T) -> Option<usize> {
        self.map.remove(element)
    }

    /// Removes an element from the set, if it exists, returning
    /// both the value that compared equal and the index at which
    /// it was stored.
    #[must_use = "Use remove if the value is not needed."]
    pub fn take(&mut self, element: &T) -> Option<(usize, T)> {
        self.map.take(element).map(|(i, v)| (i, v.0))
    }

    /// Swaps the positions of element_a with the position of element_b
    ///
    /// Returns true if both elements were found and succesfully swapped.
    pub fn swap(&mut self, element_a: &T, element_b: &T) -> bool {
        self.map.swap(element_a, element_b)
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
        element_iter: I,
    ) -> Result<Self, CapacityError<(Self, T)>> {
        let iter_for_map = element_iter.into_iter().map(|e| (e, ()));

        match PetitMap::try_from_iter(iter_for_map) {
            Ok(map) => Ok(PetitSet { map }),
            Err(CapacityError((map, failed_value))) => {
                Err(CapacityError((PetitSet { map }, failed_value.0)))
            }
        }
    }

    /// Construct a [`PetitSet`] directly from an array, without checking for duplicates.
    ///
    /// It is a logic error if any two non-`None` values in the array are equal, as elements are expected to be unique.
    /// If this occurs, the [`PetitSet`] returned may behave unpredictably.
    pub fn from_raw_array_unchecked(values: [Option<T>; CAP]) -> Self {
        // Convert from Option<T> to the required Option<(T, ())>
        let values_for_map = values.map(|v| v.map(|v| (v, ())));

        Self {
            map: PetitMap::from_raw_array_unchecked(values_for_map),
        }
    }
}

impl<T: Eq, const CAP: usize> FromIterator<T> for PetitSet<T, CAP> {
    /// Panics if the iterator contains more than `CAP` distinct elements.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        PetitSet::try_from_iter(iter).unwrap()
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

/// The `Ok` result of a successful [`PetitSet`] insertion operation
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SuccesfulSetInsertion {
    /// This is a new element: it is stored at the provided index
    NovelElenent(usize),
    /// This element was already in the set: it is stored at the provided index
    ExtantElement(usize),
}
