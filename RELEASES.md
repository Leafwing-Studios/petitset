# Release Notes

## Version 0.2

- added a `std` feature, which is enabled when standard-library support is needed
- renamed `thiserror_trait` feature to `thiserror`
- added serialization and deserialization support with `serde`

## Version 0.1.2

- `PetitSet` and `PetitMap` are now hashable if the underlying types are hashable.

## Version 0.1.1

- remove pointless `Copy` bound on `PetitMap::default()`
- forbid `unsafe` code (thanks to @5225225 for the help!)

## Version 0.1

- Released!
