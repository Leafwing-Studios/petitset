# About

Array-backed ordered set and map data structures in Rust, optimized for stack-allocated storage of a tiny number of elements with a fixed cap.
Your elements only need `Eq`, and this crate is both entirely safe and `no_std` compatible!

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

## Contributing

This repository is open to community contributions!
*Leafwing Studios* attempts to adhere to the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html).
If you haven't seen it before, it's an excellent resource!

There are a few options if you'd like to help:

1. File issues for bugs you find or new features you'd like.
2. Read over and discuss issues, then make a PR that fixes them. Use "Fixes #X" in your PR description to automatically close the issue when the PR is merged.
3. Review existing PRs, and leave thoughtful feedback. If you think a PR is ready to merge, hit "Approve" in your review!

Any contributions made are provided under the license(s) listed in this repo at the time of their contribution, and do not require separate attribution.

### Testing

1. Use doc tests aggressively to show how APIs should be used.
You can use `#` to hide a setup line from the doc tests.
2. Unit test belong near the code they are testing. Use `#[cfg(test)]` on the test module to ignore it during builds, and `#[test]` on the test functions to ensure they are run.
3. Integration tests should be stored in the top level `tests` folder, importing functions from `lib.rs`.

Use `cargo test` to run all tests.

### CI

The CI will:

1. Ensure the code is formatted with `cargo fmt`.
2. Ensure that the code compiles.
3. Ensure that (almost) all `clippy` lints pass.
4. Ensure all tests pass on Windows, MacOS and Ubuntu.

Check this locally with:

1. `cargo run -p ci`
2. `cargo test --workspace`

To manually rerun CI:

1. Navigate to the `Actions` tab.
2. Use the dropdown menu in the CI run of interest and select "View workflow file".
3. In the top-right corner, select "Rerun workflow".

### Documentation

Reference documentation is handled with standard Rust doc strings.
Use `cargo doc --open` to build and then open the docs.

Design docs (or other book-format documentation) is handled with [mdBook](https://rust-lang.github.io/mdBook/index.html).
Install it with `cargo install mdbook`, then use `mdbook serve --open` to launch the docs.
