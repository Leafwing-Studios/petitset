mod predicates;
use predicates::is_sorted;

use petitset::{CapacityError, PetitSet, SuccesfulSetInsertion};

#[test]
fn reject_duplicates() {
    let mut set: PetitSet<u8, 4> = PetitSet::default();
    assert!(set.is_empty());

    set.insert(1);
    assert!(set.len() == 1);

    set.insert(1);
    assert!(set.len() == 1);

    let result = set.insert(1);
    assert_eq!(result, SuccesfulSetInsertion::ExtantElement(0));
    assert!(set.len() == 1);

    set.insert_at(1, 0);
    assert!(set.len() == 1);

    set.insert_at(1, 1);
    assert!(set.len() == 1);
}

#[test]
fn reject_overfull() {
    let mut set: PetitSet<u8, 2> = PetitSet::default();

    set.extend(1..=2);
    assert!(set.len() == set.capacity());

    // Duplicates do not overflow
    let duplicate_result = set.try_insert(2);
    assert_eq!(
        duplicate_result,
        Ok(SuccesfulSetInsertion::ExtantElement(1))
    );
    assert!(set.len() == set.capacity());

    // Non-duplicates fail to insert
    let overfull_result = set.try_insert(3);
    assert_eq!(overfull_result, Err(CapacityError(3)));
    assert!(set.len() == set.capacity());
}

#[test]
#[should_panic]
fn panic_on_overfull_insertion() {
    let mut set: PetitSet<u8, 2> = PetitSet::default();

    set.extend(1..=2);
    assert!(set.len() == set.capacity());

    set.insert(3);
}

#[test]
fn in_order_iteration() {
    let mut set: PetitSet<u8, 8> = PetitSet::default();
    set.extend(0..8);
    assert!(is_sorted(&set));

    set.remove_at(3);
    assert!(is_sorted(&set));

    set.remove(&5);
    assert!(is_sorted(&set));

    set.remove_at(0);
    assert!(is_sorted(&set));

    set.remove_at(7);
    assert!(is_sorted(&set));

    let mut backwards_set: PetitSet<u8, 8> = PetitSet::default();
    backwards_set.extend((0..8).rev());
    assert!(!is_sorted(&backwards_set));
}

#[test]
fn equality_ignores_order() {
    let mut set_1: PetitSet<u8, 16> = PetitSet::default();
    set_1.extend(7..=11);

    let set_2: PetitSet<u8, 16> = PetitSet::try_from_iter((7..=11).rev()).unwrap();
    assert_eq!(set_1, set_2);
}

#[test]
fn removal_returns_items() {
    let mut set: PetitSet<u8, 8> = PetitSet::default();
    set.extend(0..8);

    let index = set.remove(&3).unwrap();
    assert_eq!(index, 3);

    let value = set.take_at(5).unwrap();
    assert_eq!(value, 5);
}

#[test]
fn remove_and_insert_in_same_place() {
    let mut set: PetitSet<u8, 8> = PetitSet::default();
    set.extend(0..8);
    assert!(is_sorted(&set));

    set.remove(&3);
    assert!(is_sorted(&set));

    set.insert(3);
    assert!(is_sorted(&set));

    set.remove_at(5);
    assert!(set.get_at(5).is_none());

    set.insert_at(5, 5);
    assert!(is_sorted(&set));
}

#[test]
fn hashable() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut set_1: PetitSet<u8, 8> = PetitSet::default();
    set_1.insert(42);

    let mut set_2: PetitSet<u8, 8> = PetitSet::default();
    set_2.insert(42);

    let mut set_3: PetitSet<u8, 8> = PetitSet::default();
    set_3.insert_at(42, 3);

    let mut set_4: PetitSet<u8, 8> = PetitSet::default();
    set_4.insert(43);

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    assert_eq!(calculate_hash(&set_1), calculate_hash(&set_1));
    assert_eq!(calculate_hash(&set_1), calculate_hash(&set_2));
    // Hashes are sensitive to slot
    assert!(calculate_hash(&set_1) != calculate_hash(&set_3));
    // Hashes are sensitive to element value
    assert!(calculate_hash(&set_1) != calculate_hash(&set_4));
}
