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
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use core::{
    cmp::max,
    fmt,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[cfg(feature = "std")]
use std::{
    cmp::max,
    fmt,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

pub use crate::{
    error::NodeError,
    id::NodeId,
    traverse::{
        Ancestors, Children, Descendants, FollowingSiblings, NodeEdge,
        PrecedingSiblings, ReverseChildren, ReverseTraverse, Traverse,
    },
};

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

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "derive-eq", derive(Eq))]
/// An `Arena` structure containing certain Nodes
pub struct Arena<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Arena<T> {
    /// Create a new empty `Arena`
    pub fn new() -> Arena<T> {
        Self::default()
    }

    /// Create a new node from its associated data.
    ///
    /// # Panics
    ///
    /// Panics if the arena already has `usize::max_value()` nodes.
    pub fn new_node(&mut self, data: T) -> NodeId {
        let next_index1 = NonZeroUsize::new(self.nodes.len().wrapping_add(1))
            .expect("Too many nodes in the arena");
        self.nodes.push(Node {
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            removed: false,
            data,
        });
        NodeId::from_non_zero_usize(next_index1)
    }

    // Count nodes in arena.
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    // Returns true if arena has no nodes, false otherwise
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Get a reference to the node with the given id if in the arena, None
    /// otherwise.
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.index0())
    }

    /// Get a mutable reference to the node with the given id if in the arena,
    /// None otherwise.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.index0())
    }

    /// Iterate over all nodes in the arena in storage-order.
    ///
    /// Note that this iterator also contains removed elements, which can be
    /// tested with the `is_removed()` method on the node.
    pub fn iter(&self) -> Iter<Node<T>> {
        self.nodes.iter()
    }
}

#[cfg(feature = "par_iter")]
impl<T: Sync> Arena<T> {
    /// Return an parallel iterator over the whole arena.
    ///
    /// Note that this iterator also contains removed elements, which can be
    /// tested with the `is_removed()` method on the node.
    pub fn par_iter(&self) -> rayon::slice::Iter<Node<T>> {
        self.nodes.par_iter()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
}

trait GetPairMut<T> {
    /// Get mutable references to two distinct nodes
    fn get_tuple_mut(&mut self, a: usize, b: usize)
        -> Option<(&mut T, &mut T)>;
}

impl<T> GetPairMut<T> for Vec<T> {
    fn get_tuple_mut(
        &mut self,
        a: usize,
        b: usize,
    ) -> Option<(&mut T, &mut T)> {
        if a == b {
            return None;
        }
        let (xs, ys) = self.split_at_mut(max(a, b));
        if a < b {
            Some((&mut xs[a], &mut ys[0]))
        } else {
            Some((&mut ys[0], &mut xs[b]))
        }
    }
}

impl<T> Index<NodeId> for Arena<T> {
    type Output = Node<T>;

    fn index(&self, node: NodeId) -> &Node<T> {
        &self.nodes[node.index0()]
    }
}

impl<T> IndexMut<NodeId> for Arena<T> {
    fn index_mut(&mut self, node: NodeId) -> &mut Node<T> {
        &mut self.nodes[node.index0()]
    }
}
