# indextree

[![Build status](https://circleci.com/gh/saschagrunert/indextree.svg?style=shield)](https://circleci.com/gh/saschagrunert/indextree)
[![Build Status](https://travis-ci.org/saschagrunert/indextree.svg)](https://travis-ci.org/saschagrunert/indextree)
[![Build status](https://ci.appveyor.com/api/projects/status/byraapuh9py02us0?svg=true)](https://ci.appveyor.com/project/saschagrunert/indextree)
[![Coverage](https://codecov.io/gh/saschagrunert/indextree/branch/master/graph/badge.svg)](https://codecov.io/gh/saschagrunert/indextree)
[![dependency status](https://deps.rs/repo/github/saschagrunert/indextree/status.svg)](https://deps.rs/repo/github/saschagrunert/indextree)
[![doc indextree](https://img.shields.io/badge/master_doc-indextree-blue.svg)](https://saschagrunert.github.io/indextree)
[![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/saschagrunert/indextree/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/indextree.svg)](https://crates.io/crates/indextree)
[![doc.rs](https://docs.rs/indextree/badge.svg)](https://docs.rs/indextree)

## Arena based tree structure with multithreading support

This arena tree structure is using just a single `Vec` and numerical identifiers
(indices in the vector) instead of reference counted pointers. This means there
is no `RefCell` and mutability is handled in a way much more idiomatic to Rust
through unique (&mut) access to the arena. The tree can be sent or shared across
threads like a `Vec`. This enables general multiprocessing support like
parallel tree traversals.

### Example usage

```rust
use indextree::Arena;

// Create a new arena
let arena = &mut Arena::new();

// Add some new nodes to the arena
let a = arena.new_node(1);
let b = arena.new_node(2);

// Append a to b
assert!(a.append(b, arena).is_ok());
assert_eq!(b.ancestors(arena).into_iter().count(), 2);
```
