//! A module for the [`PetitSet`] data structure

/// A set-like data structure with a fixed maximum size
///
/// This data structure does not require the [`Hash`] or [`Ord`] traits,
/// and instead uses linear iteration to find entries.
/// Iteration order is guaranteed to be stable, and elements are not re-compressed upon removal.
///
/// In almost all cases, you will want to ensure that `T` is both [`Copy`] and [`Eq`].
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
#[derive(Debug, Clone, Copy)]
pub struct PetitSet<T, const CAP: usize> {
    storage: [Option<T>; CAP],
}

// No bounds
impl<T, const CAP: usize> PetitSet<T, CAP> {
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

    /// Returns the current number of elements in the [`PetitSet`]
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage.iter().filter(|e| e.is_some()).count()
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
}

// Only Copy
impl<T: Copy, const CAP: usize> PetitSet<T, CAP> {
    /// Create a new empty [`PetitSet`] where all values are the same
    ///
    /// The capacity is given by the generic parameter `CAP`.
    #[must_use]
    pub fn new(value: T) -> Self {
        PetitSet {
            storage: [Some(value); CAP],
        }
    }

    /// Removes all elements from the set without allocation
    pub fn clear(&mut self) {
        self.storage = [None; CAP];
    }

    /// Removes the element at the provided index
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    ///
    /// PANICS: panics if the provided index is larger than CAP.
    pub fn remove_at(&mut self, index: usize) -> Option<T> {
        assert!(index <= CAP);

        let removed = self.storage[index];
        self.storage[index] = None;
        removed
    }

    /// Returns a mutable reference to the value at the provided index of the underlying array
    ///
    /// PANICS: panics if the index is out-of-bounds or does not contain data
    #[must_use]
    pub fn get_unchecked(&mut self, index: usize) -> T {
        assert!(index <= CAP);
        self.storage[index].unwrap()
    }
}

impl<T: Copy, const CAP: usize> Default for PetitSet<T, CAP> {
    fn default() -> Self {
        Self {
            storage: [None; CAP],
        }
    }
}

// Only Eq
impl<T: Eq, const CAP: usize> PetitSet<T, CAP> {
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
    /// Returns Ok if this succeeds, or an error if this failed due to either capacity or a duplicate entry.
    pub fn try_insert(&mut self, element: T) -> Result<(), InsertionError> {
        if self.contains(&element) {
            return Err(InsertionError::Duplicate);
        }

        if let Some(index) = self.next_empty_index(0) {
            self.storage[index] = Some(element);
            Ok(())
        } else {
            Err(InsertionError::Overfull)
        }
    }

    /// Insert a new element to the set
    ///
    /// PANICS: will panic if the set is full and the item is not a duplicate
    pub fn insert(&mut self, element: T) {
        // Always insert
        if let Err(InsertionError::Overfull) = self.try_insert(element) {
            // But panic if the set was full
            panic!("Inserting this element would have overflowed the set!")
        }
    }

    /// Inserts multiple new elements to the set
    ///
    /// PANICS: will panic if the set would overflow due to the insertion of non-duplicate items
    pub fn insert_multiple(&mut self, elements: impl IntoIterator<Item = T>) {
        for element in elements {
            self.insert(element);
        }
    }
}

// Copy and Eq
impl<T: Copy + Eq, const CAP: usize> PetitSet<T, CAP> {
    /// Removes the element from the set, if it exists
    ///
    /// Returns `Some(index, T)` for the first matching element found, or `None` if no matching element is found
    pub fn remove(&mut self, element: &T) -> Option<(usize, T)> {
        if let Some(index) = self.find(element) {
            let removed_element = self.remove_at(index).unwrap();
            Some((index, removed_element))
        } else {
            None
        }
    }

    /// Insert a new element to the set at the provided index
    ///
    /// Returns `Some(T)` if an element was found at that index, or `None` if no element was there.
    /// If a matching element already exists in the set, `None` will be returned.
    ///
    /// PANICS: panics if the provided index is larger than CAP.
    pub fn insert_at(&mut self, element: T, index: usize) -> Option<T> {
        assert!(index <= CAP);

        if self.contains(&element) {
            return None;
        }

        let preexisting_element = self.remove_at(index);
        self.storage[index] = Some(element);

        preexisting_element
    }
}

/// An [`Iterator`] struct for [`PetitSet`]
#[derive(Default, Clone, Copy, Debug)]
pub struct PetitSetIter<T: Copy, const CAP: usize> {
    set: PetitSet<T, CAP>,
    cursor: usize,
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> Iterator for PetitSetIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.set.next_filled_index(self.cursor) {
            self.cursor = index + 1;
            Some(self.set.get_unchecked(index))
        } else {
            self.cursor = CAP;
            None
        }
    }
}

impl<T: PartialEq + Clone + Copy, const CAP: usize> IntoIterator for PetitSet<T, CAP> {
    type Item = T;
    type IntoIter = PetitSetIter<T, CAP>;
    fn into_iter(self) -> Self::IntoIter {
        PetitSetIter {
            set: self,
            cursor: 0,
        }
    }
}

impl<T: Eq + Copy, const CAP: usize> FromIterator<T> for PetitSet<T, CAP> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set: PetitSet<T, CAP> = PetitSet::default();
        for element in iter {
            set.insert(element);
        }
        set
    }
}

impl<T: Eq + Copy, const CAP: usize> PartialEq for PetitSet<T, CAP> {
    /// Uses an inefficient O(n^2) approach to avoid introducing additional trait bounds
    fn eq(&self, other: &Self) -> bool {
        // Two sets cannot be equal if their cardinality differs
        if self.len() != other.len() {
            return false;
        }

        for item in *self {
            let mut match_found = false;
            for other_item in *other {
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

/// An error returned when attempting to insert into a [`PetitSet`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InsertionError {
    /// The set was full before insertion was attempted
    Overfull,
    /// A matching entry already existed
    Duplicate,
}
