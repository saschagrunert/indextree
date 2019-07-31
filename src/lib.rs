//! # Arena based tree data structure
//!
//! This arena tree structure is using just a single `Vec` and numerical
//! identifiers (indices in the vector) instead of reference counted pointers
//! like. This means there is no `RefCell` and mutability is handled in a way
//! much more idiomatic to Rust through unique (&mut) access to the arena. The
//! tree can be sent or shared across threads like a `Vec`. This enables
//! general multiprocessing support like parallel tree traversals.  
//! # Example usage
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
//! assert!(a.append(b, arena).is_ok());
//! assert_eq!(b.ancestors(arena).into_iter().count(), 2);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::fmt;

use crate::arena::GetPairMut;
pub use crate::{
    arena::Arena,
    error::NodeError,
    id::NodeId,
    traverse::{
        Ancestors, Children, Descendants, FollowingSiblings, NodeEdge,
        PrecedingSiblings, ReverseChildren, ReverseTraverse, Traverse,
    },
};

mod arena;
mod error;
mod id;
mod traverse;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "derive-eq", derive(Eq))]
/// A node within a particular `Arena`
pub struct Node<T> {
    // Keep these private (with read-only accessors) so that we can keep them
    // consistent. E.g. the parent of a node’s child is that node.
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    removed: bool,

    /// The actual data which will be stored within the tree
    pub data: T,
}

impl<T> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "parent: {}; ", parent)?;
        } else {
            write!(f, "no parent; ")?;
        }
        if let Some(previous_sibling) = self.previous_sibling {
            write!(f, "previous sibling: {}; ", previous_sibling)?;
        } else {
            write!(f, "no previous sibling; ")?;
        }
        if let Some(next_sibling) = self.next_sibling {
            write!(f, "next sibling: {}; ", next_sibling)?;
        } else {
            write!(f, "no next sibling; ")?;
        }
        if let Some(first_child) = self.first_child {
            write!(f, "first child: {}; ", first_child)?;
        } else {
            write!(f, "no first child; ")?;
        }
        if let Some(last_child) = self.last_child {
            write!(f, "last child: {}; ", last_child)?;
        } else {
            write!(f, "no last child; ")?;
        }
        Ok(())
    }
}
