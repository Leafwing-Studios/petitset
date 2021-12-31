mod tests {
    use petitset::set::*;

    #[test]
    fn reject_duplicates() {
        let mut set: PetitSet<u8, 4> = PetitSet::default();
        assert!(set.len() == 0);

        set.insert(1);
        assert!(set.len() == 1);

        set.insert(1);
        assert!(set.len() == 1);

        let result = set.try_insert(1);
        assert_eq!(result, Err(InsertionError::Duplicate));
        assert!(set.len() == 1);

        set.insert_at(1, 0);
        assert!(set.len() == 1);

        set.insert_at(1, 1);
        assert!(set.len() == 1);
    }

    #[test]
    fn reject_overfull() {
        let mut set: PetitSet<u8, 2> = PetitSet::default();

        set.insert_multiple(1..=2);
        assert!(set.len() == set.capacity());

        // Duplicates do not overflow
        let duplicate_result = set.try_insert(1);
        assert_eq!(duplicate_result, Err(InsertionError::Duplicate));
        assert!(set.len() == set.capacity());

        // Non-duplicates fail to insert
        let overfull_result = set.try_insert(3);
        assert_eq!(overfull_result, Err(InsertionError::Overfull));
        assert!(set.len() == set.capacity());
    }

    #[test]
    #[should_panic]
    fn panic_on_overfull_insertion() {
        let mut set: PetitSet<u8, 2> = PetitSet::default();

        set.insert_multiple(1..=2);
        assert!(set.len() == set.capacity());

        set.insert(3);
    }

    #[test]
    fn in_order_iteration() {
        let mut set: PetitSet<u8, 8> = PetitSet::default();
        set.insert_multiple(0..8);
        assert!(set.is_sorted());

        set.remove_at(3);
        assert!(set.is_sorted());

        set.remove(&5);
        assert!(set.is_sorted());

        set.remove_at(0);
        assert!(set.is_sorted());

        set.remove_at(7);
        assert!(set.is_sorted());

        let mut backwards_set: PetitSet<u8, 8> = PetitSet::default();
        backwards_set.insert_multiple(8..0);
        assert!(!backwards_set.is_sorted());
    }

    #[test]
    fn equality_ignores_order() {
        let mut set_1: PetitSet<u8, 16> = PetitSet::default();
        set_1.insert_multiple(7..=11);

        let set_2: PetitSet<u8, 16> = PetitSet::from_iter(11..=7);
        assert_eq!(set_1, set_2);
    }

    #[test]
    fn removal_returns_items() {
        let mut set: PetitSet<u8, 8> = PetitSet::default();
        set.insert_multiple(0..8);

        let (index, value) = set.remove(&3).unwrap();
        assert_eq!(index, 3);
        assert_eq!(value, 3);

        let value = set.remove_at(5).unwrap();
        assert_eq!(value, 5);
    }

    #[test]
    fn remove_and_insert_in_same_place() {
        let mut set: PetitSet<u8, 8> = PetitSet::default();
        set.insert_multiple(0..8);
        assert!(set.is_sorted());

        set.remove(&3);
        assert!(set.is_sorted());

        set.insert(3);
        assert!(set.is_sorted());

        set.remove_at(5);
        assert!(set.get(5).is_none());

        set.insert_at(5, 5);
        assert!(set.is_sorted());
    }
}
