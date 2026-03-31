//! # Arena based tree data structure
//!
//! This arena tree structure is using just a single `Vec` and numerical
//! identifiers (indices in the vector) instead of reference counted pointers.
//! This means there is no `RefCell` and mutability is handled in a way
//! much more idiomatic to Rust through unique (&mut) access to the arena. The
//! tree can be sent or shared across threads like a `Vec`. This enables
//! general multiprocessing support like parallel tree traversals.
//!
//! # Features
//!
//! * `std` (default) - Enable standard library support. Disable for `no_std`
//!   environments (requires `alloc`).
//! * `macros` (default) - Enable the `tree!` macro for declarative tree
//!   construction.
//! * `deser` - Enable `serde` serialization and deserialization for
//!   [`Arena`], [`Node`], and [`NodeId`].
//! * `par_iter` - Enable parallel iteration via `Arena::par_iter()` using
//!   [rayon](https://docs.rs/rayon).
//!
//! # Node removal and reuse
//!
//! Calling [`NodeId::remove`] does not deallocate the node slot. Instead, it
//! marks the slot for reuse via an internal generation counter (stamp). Future
//! calls to [`Arena::new_node`] may recycle freed slots. Stale [`NodeId`]
//! references are detected through [`NodeId::is_removed`], which compares
//! the ID's stamp against the current slot stamp.
//!
//! # Example usage
//!
//! ```
//! use indextree::Arena;
//!
//! // Create a new arena
//! let arena = &mut Arena::new();
//!
//! // Add some new nodes to the arena
//! let a = arena.new_node(1);
//! let b = arena.new_node(2);
//!
//! // Append b to a
//! a.append(b, arena);
//! assert_eq!(b.ancestors(arena).count(), 2);
//! ```
//!
//! # Error handling
//!
//! Methods that modify the tree come in panicking and checked variants.
//! The checked variants (e.g. [`NodeId::checked_append`]) return a
//! [`Result`] with a [`NodeError`] on failure, while the panicking
//! variants (e.g. [`NodeId::append`]) call `.expect()` internally.
//!
//! ```
//! use indextree::{Arena, NodeError};
//!
//! let mut arena = Arena::new();
//! let root = arena.new_node("root");
//!
//! // Cannot append a node to itself
//! assert!(matches!(
//!     root.checked_append(root, &mut arena),
//!     Err(NodeError::AppendSelf)
//! ));
//! ```
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub use crate::{
    arena::Arena,
    debug_pretty_print::DebugPrettyPrint,
    error::NodeError,
    id::NodeId,
    node::Node,
    traverse::{
        Ancestors, Children, Descendants, FollowingSiblings, NodeEdge, PrecedingSiblings,
        Predecessors, ReverseTraverse, Traverse,
    },
};

#[cfg(feature = "macros")]
pub use indextree_macros as macros;

// Compile-time assertions that Arena and NodeId are Send + Sync.
#[allow(dead_code)]
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assertions() {
        assert_send_sync::<Arena<u32>>();
        assert_send_sync::<NodeId>();
    }
};

#[macro_use]
pub(crate) mod relations;

mod arena;
mod debug_pretty_print;
pub(crate) mod error;
mod id;
mod node;
pub(crate) mod siblings_range;
mod traverse;
