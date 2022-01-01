mod predicates;
use predicates::is_sorted;

use petitset::{CapacityError, PetitSet};

#[test]
fn reject_duplicates() {
    let mut set: PetitSet<u8, 4> = PetitSet::default();
    assert!(set.is_empty());

    set.insert(1);
    assert!(set.len() == 1);

    set.insert(1);
    assert!(set.len() == 1);

    let result = set.insert(1);
    assert_eq!(result, (0, false));
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
    assert_eq!(duplicate_result, Ok((1, false)));
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
