//! Algebraic manipulations of `PetitSets`
use crate::set::{PetitSet, PetitSetIter};

impl<T: Eq + Clone, const CAP: usize> PetitSet<T, CAP> {
    /// Returns an iterator of references to the values that are in `self` but not in `other`.
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([7, 13, 5]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([15, 7, 3, 4, 5]);
    ///  
    /// let set_a_minus_b: PetitSet<usize, 3> = PetitSet::from_iter([13]);
    /// let set_b_minus_a: PetitSet<usize, 5> = PetitSet::from_iter([15, 3, 4]);
    ///
    /// let computed_set_a_minus_b = set_a.difference(&set_b).into_set();
    /// let computed_set_b_minus_a = set_b.difference(&set_a).into_set();
    ///
    /// assert_eq!(set_a_minus_b, computed_set_a_minus_b);
    /// assert_eq!(set_b_minus_a, computed_set_b_minus_a);
    /// ```
    pub fn difference<const OTHER_CAP: usize>(
        &self,
        other: &PetitSet<T, OTHER_CAP>,
    ) -> PetitSetIter<T, CAP> {
        let mut iter: PetitSetIter<T, CAP> = PetitSetIter::default();
        for s in self.iter() {
            if !other.contains(s) {
                iter.set.insert_unchecked(s.clone());
            }
        }

        iter
    }

    /// Returns an iterator of references to the values that are not in both `self` and `other`.
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([7, 13, 5]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([15, 7, 3, 4, 5]);
    ///  
    /// let set_a_sym_diff_b: PetitSet<usize, 8> = PetitSet::from_iter([13, 15, 3, 4]);
    ///
    /// let computed_set_a_sym_diff_b = set_a.symmetric_difference(&set_b).into_set();
    /// let computed_set_b_sym_diff_a = set_b.symmetric_difference(&set_a).into_set();
    ///
    /// assert_eq!(set_a_sym_diff_b, computed_set_a_sym_diff_b);
    /// assert_eq!(computed_set_a_sym_diff_b, computed_set_b_sym_diff_a);
    /// ```
    pub fn symmetric_difference<const OTHER_CAP: usize>(
        &self,
        other: &PetitSet<T, OTHER_CAP>,
    ) -> PetitSetIter<T, { CAP + OTHER_CAP }> {
        let mut iter: PetitSetIter<T, { CAP + OTHER_CAP }> = PetitSetIter::default();
        for s in self.iter() {
            if !other.contains(s) {
                iter.set.insert_unchecked(s.clone());
            }
        }

        for o in other.iter() {
            if !self.contains(o) {
                iter.set.insert_unchecked(o.clone());
            }
        }

        iter
    }

    /// Returns an iterator of references to the values that are in both `self` and `other`.
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([7, 13, 5]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([15, 7, 3, 4, 5]);
    ///  
    /// let set_a_intersection_b: PetitSet<usize, 5> = PetitSet::from_iter([7, 5]);
    ///
    /// let computed_set_a_intersection_b = set_a.intersection(&set_b).into_set();
    /// let computed_set_b_intersection_a = set_b.intersection(&set_a).into_set();
    ///
    /// assert_eq!(set_a_intersection_b, computed_set_a_intersection_b);
    /// assert_eq!(computed_set_a_intersection_b, computed_set_b_intersection_a);
    /// ```
    pub fn intersection<const OTHER_CAP: usize>(
        &self,
        other: &PetitSet<T, OTHER_CAP>,
    ) -> PetitSetIter<T, { max_of(CAP, OTHER_CAP) }> {
        let mut iter: PetitSetIter<T, { max_of(CAP, OTHER_CAP) }> = PetitSetIter::default();
        for s in self.iter() {
            if other.contains(s) {
                iter.set.insert_unchecked(s.clone());
            }
        }
        iter
    }

    /// Returns an iterator of references to the values that are in either `self` and `other`.
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([7, 13, 5]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([15, 7, 3, 4, 5]);
    ///  
    /// let set_a_union_b: PetitSet<usize, 8> = PetitSet::from_iter([7, 13, 5, 15, 3, 4]);
    ///
    /// let computed_set_a_union_b = set_a.union(&set_b).into_set();
    /// let computed_set_b_union_a = set_b.union(&set_a).into_set();
    ///
    /// assert_eq!(set_a_union_b, computed_set_a_union_b);
    /// assert_eq!(computed_set_a_union_b, computed_set_b_union_a);
    /// ```
    pub fn union<const OTHER_CAP: usize>(
        &self,
        other: &PetitSet<T, OTHER_CAP>,
    ) -> PetitSetIter<T, { CAP + OTHER_CAP }> {
        let mut iter: PetitSetIter<T, { CAP + OTHER_CAP }> = PetitSetIter::default();
        for s in self.iter() {
            iter.set.insert_unchecked(s.clone());
        }

        for o in other.iter() {
            // We are not guaranteed uniqueness by construction here
            iter.set.insert(o.clone());
        }

        iter
    }

    /// Do the sets contain any common elements?
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([7, 13, 5]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([15, 7, 3, 4, 5]);
    /// let mut set_c: PetitSet<usize, 1> = PetitSet::default();
    /// set_c.insert(42);
    ///
    /// assert!(!set_a.is_disjoint(&set_b));
    /// assert!(!set_b.is_disjoint(&set_a));
    ///
    /// assert!(set_a.is_disjoint(&set_c));
    /// assert!(set_c.is_disjoint(&set_a));
    /// ```
    pub fn is_disjoint<const OTHER_CAP: usize>(&self, other: &PetitSet<T, OTHER_CAP>) -> bool {
        for s in self.iter() {
            for o in other.iter() {
                if s == o {
                    return false;
                }
            }
        }
        true
    }

    /// Are all elements in `self` contained in `other`?
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([1, 2, 3]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([2, 3]);
    ///
    /// assert!(set_a.is_subset(&set_a));
    ///
    /// assert!(!set_a.is_subset(&set_b));
    /// assert!(set_b.is_subset(&set_a));
    /// ```
    pub fn is_subset<const OTHER_CAP: usize>(&self, other: &PetitSet<T, OTHER_CAP>) -> bool {
        'outer: for s in self.iter() {
            '_inner: for o in other.iter() {
                if s == o {
                    // If we've found a match in other, check the next element
                    continue 'outer;
                }
            }
            // If no match could be found, there is an element in self that is not in other
            return false;
        }
        true
    }

    /// Are all elements in `other` contained in `self`?
    ///
    /// # Examples
    /// ```rust
    /// use petitset::PetitSet;
    ///
    /// let set_a: PetitSet<usize, 3> = PetitSet::from_iter([1, 2, 3]);
    /// let set_b: PetitSet<usize, 5> = PetitSet::from_iter([2, 3]);
    ///
    /// assert!(set_a.is_superset(&set_a));
    ///
    /// assert!(set_a.is_superset(&set_b));
    /// assert!(!set_b.is_superset(&set_a));
    /// ```
    pub fn is_superset<const OTHER_CAP: usize>(&self, other: &PetitSet<T, OTHER_CAP>) -> bool {
        'outer: for o in other.iter() {
            '_inner: for s in self.iter() {
                if o == s {
                    // If we've found a match in self, check the next element
                    continue 'outer;
                }
            }
            // If no match could be found, there is an element in other that is not in self
            return false;
        }
        true
    }
}

/// Trivial const replacement for `std::comp::Ord::max`
pub const fn max_of(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}
