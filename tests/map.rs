mod tests {
    use petitset::PetitMap;

    #[test]
    fn lookup() {
        let mut map: PetitMap<i32, i32, 4> = PetitMap::default();
        // Index 0
        map.insert(1, 11);
        // Index 1
        map.insert(3, 31);
        // Index 2
        map.insert(4, 41);
        // Index 3
        map.insert(2, 21);

        assert_eq!(*map.get(&1).unwrap(), 11);
        assert_eq!(*map.get_mut(&3).unwrap(), 31);
        assert_eq!(map.get_at(2).unwrap(), (4, 41));
        assert_eq!(map.get_at_mut(3).unwrap(), (&mut 2, &mut 21));
    }

    #[test]
    fn removal() {
        let mut map: PetitMap<i32, i32, 4> = PetitMap::default();
        // Index 0
        map.insert(1, 11);
        // Index 1
        map.insert(3, 31);
        // Index 2
        map.insert(4, 41);
        // Index 3
        map.insert(2, 21);

        // Overwriting insertion
        map.insert(3, 33);

        let removed = map.remove(&3);
        assert_eq!(removed, Some((1, 3, 33)));
        assert_eq!(map.find(&3), None);

        let removed_at = map.remove_at(0);
        assert_eq!(removed_at, Some((1, 11)));
        assert_eq!(map.find(&1), None);

        let failed_remove = map.remove_at(0);
        assert_eq!(failed_remove, None);
    }

    #[test]
    #[should_panic]
    fn panic_on_overfull_insertion() {
        let mut map: PetitMap<i32, i32, 2> = PetitMap::default();
        map.insert(1, 1);
        map.insert(2, 2);
        map.insert(3, 3);
    }
}
