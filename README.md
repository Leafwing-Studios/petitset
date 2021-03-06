# About

Array-backed ordered set and map data structures in Rust, optimized for stack-allocated storage of a tiny number of elements with a fixed cap.
All you need is `Eq`!

This crate is:

- entirely safe
- fully documented
- `no_std` compatible
- zero dependencies (unless you want `thiserror` or `serde` integration)

This crate is designed to be used in performance-sensitive contexts with a small number of elements, where iteration is more common than look-ups and you don't mind a fixed size.
One particularly useful quirk is that elements are not recompacted upon removal: this can be very useful when replacing elements in a set or using the indexes that the elements are stored at in a semantic fashion.
Iteration order is guaranteed to be stable, on a first-in-first-out basis.

If this isn't what you're after, check out one of these alternatives!

- [smolset](https://crates.io/crates/smolset): automatically converts to a `HashSet` when the number of elements is too large. Unordered.
- [array_map](https://docs.rs/array_map/latest/array_map/index.html): Blazing fast, fixed size. All possible keys must be known statically.
- [sparseset](https://github.com/bombela/sparseset): Heap-allocated, great for sparse data and frequent iteration. Stable order!
- [HashSet](https://doc.rust-lang.org/std/collections/struct.HashSet.html): Heap-allocated, unordered, requires `Hash`, unbounded size.
- [BTreeSet](https://doc.rust-lang.org/stable/std/collections/struct.BTreeSet.html): Heap-allocated, ordered, requires `Ord`, unbounded size.
- [IndexMap](https://docs.rs/indexmap/latest/indexmap/): Heap-allocated, requires `Hash`, unbounded size.

This crate has a reasonable collection of convenience methods for working with both sets and maps, aiming for rough equivalence with `HashMap` and `HashSet`.
If you'd like more, please submit an issue or PR!
