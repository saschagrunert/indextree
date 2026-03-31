# indextree

[![GitHub Actions](https://github.com/saschagrunert/indextree/actions/workflows/test.yml/badge.svg)](https://github.com/saschagrunert/indextree/actions/workflows/test.yml)
[![Coverage](https://codecov.io/gh/saschagrunert/indextree/branch/main/graph/badge.svg)](https://codecov.io/gh/saschagrunert/indextree)
[![Dependency Status](https://deps.rs/repo/github/saschagrunert/indextree/status.svg)](https://deps.rs/repo/github/saschagrunert/indextree)
[![Doc indextree](https://img.shields.io/badge/main-indextree-blue.svg)](https://saschagrunert.github.io/indextree/doc/indextree)
[![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/saschagrunert/indextree/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/indextree.svg)](https://crates.io/crates/indextree)
[![doc.rs](https://docs.rs/indextree/badge.svg)](https://docs.rs/indextree)

## Arena based tree structure with multithreading support

This arena tree structure is using just a single `Vec` and numerical identifiers
(indices in the vector) instead of reference counted pointers. This means there
is no `RefCell` and mutability is handled in a way much more idiomatic to Rust
through unique (&mut) access to the arena. The tree can be sent or shared across
threads like a `Vec`. This enables general multiprocessing support like
parallel tree traversals.

### Features

| Feature    | Default | Description                                                        |
| ---------- | ------- | ------------------------------------------------------------------ |
| `std`      | yes     | Standard library support. Disable for `no_std` (requires `alloc`). |
| `macros`   | yes     | `tree!` macro for declarative tree construction.                   |
| `deser`    | no      | Serde serialization and deserialization.                           |
| `par_iter` | no      | Parallel iteration via rayon.                                      |

### Example usage

```rust
use indextree::Arena;

// Create a new arena
let arena = &mut Arena::new();

// Add some new nodes to the arena
let a = arena.new_node(1);
let b = arena.new_node(2);

// Append b to a
a.append(b, arena);
assert_eq!(b.ancestors(arena).count(), 2);
```

### Building trees with the `tree!` macro

The optional `macros` feature (enabled by default) provides a `tree!` macro
for declarative tree construction:

```rust
use indextree::{Arena, macros::tree};

let arena = &mut Arena::new();
let root = tree!(arena, "root" => {
    "child_1" => {
        "grandchild_1",
        "grandchild_2",
    },
    "child_2",
    "child_3",
});

assert_eq!(root.child_count(arena), 3);
assert_eq!(root.descendants(arena).count(), 6);
```

### Building a file system tree

```rust
use indextree::Arena;

let arena = &mut Arena::new();
let root = arena.new_node("/");
let etc = root.append_value("etc/", arena);
let usr = root.append_value("usr/", arena);

etc.append_value("hosts", arena);
etc.append_value("resolv.conf", arena);

let bin = usr.append_value("bin/", arena);
bin.append_value("rustc", arena);

// Traverse and collect paths
let descendants: Vec<_> = root
    .descendants(arena)
    .map(|id| *arena[id].get())
    .collect();
assert_eq!(
    descendants,
    vec!["/", "etc/", "hosts", "resolv.conf", "usr/", "bin/", "rustc"]
);
```

### Parallel iteration

With the `par_iter` feature, trees can be traversed in parallel using
[rayon](https://crates.io/crates/rayon):

```rust,ignore
use indextree::Arena;
use rayon::prelude::*;

let arena = &mut Arena::new();
let root = arena.new_node(0);
for i in 1..=1000 {
    root.append_value(i, arena);
}

let sum: i64 = arena.par_iter().map(|node| *node.get()).sum();
assert_eq!(sum, 500500);
```
