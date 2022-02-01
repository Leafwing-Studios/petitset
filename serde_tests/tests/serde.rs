use petitset::{PetitMap, PetitSet};
use ron::{de::from_str, ser::to_string};

#[test]
fn serde_map() {
    let mut map: PetitMap<u32, &str, 5> = PetitMap::new();
    map.insert(1, "one");
    map.insert(5, "five");
    map.insert(3, "three");
    map.insert(2, "two");
    map.insert(4, "four");

    map.remove(&3);

    let serialization_result = to_string(&map);
    if let Err(serialization_error) = serialization_result {
        panic!("{serialization_error}");
    }

    let intermediate_repr = serialization_result.unwrap();
    dbg!(intermediate_repr.clone());

    let deserialization_result = from_str(&intermediate_repr);
    if let Err(deserialization_error) = deserialization_result {
        panic!("{deserialization_error}");
    }

    let deserialized_map: PetitMap<u32, &str, 5> = deserialization_result.unwrap();

    assert_eq!(map, deserialized_map);
}

#[test]
fn serde_set() {
    let mut set: PetitSet<u32, 5> = PetitSet::new();
    set.insert(5);
    set.insert(4);
    set.insert(1);
    set.insert(3);
    set.insert(2);

    set.remove(&3);

    let serialization_result = to_string(&set);
    if let Err(serialization_error) = serialization_result {
        panic!("{serialization_error}");
    }

    let intermediate_repr = serialization_result.unwrap();
    dbg!(intermediate_repr.clone());

    let deserialization_result = from_str(&intermediate_repr);
    if let Err(deserialization_error) = deserialization_result {
        panic!("{deserialization_error}");
    }

    let deserialized_set: PetitSet<u32, 5> = deserialization_result.unwrap();

    assert_eq!(set, deserialized_set);
}

#[test]
fn serde_set_string() {
    let mut set: PetitSet<String, 5> = PetitSet::new();
    set.insert("five".to_string());
    set.insert("four".to_string());
    set.insert("one".to_string());
    set.insert("three".to_string());
    set.insert("two".to_string());

    set.remove(&"three".to_string());

    let serialization_result = to_string(&set);
    if let Err(serialization_error) = serialization_result {
        panic!("{serialization_error}");
    }

    let intermediate_repr = serialization_result.unwrap();
    dbg!(intermediate_repr.clone());

    let deserialization_result = from_str(&intermediate_repr);
    if let Err(deserialization_error) = deserialization_result {
        panic!("{deserialization_error}");
    }

    let deserialized_set: PetitSet<String, 5> = deserialization_result.unwrap();

    assert_eq!(set, deserialized_set);
}
