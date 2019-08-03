//! Arena.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use core::{
    cmp::max,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{
    cmp::max,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

use crate::{Node, NodeId};

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "derive-eq", derive(Eq))]
/// An `Arena` structure containing certain [`Node`]s.
///
/// [`Node`]: struct.Node.html
pub struct Arena<T> {
    pub(crate) nodes: Vec<Node<T>>,
}

impl<T> Arena<T> {
    /// Creates a new empty `Arena`.
    pub fn new() -> Arena<T> {
        Self::default()
    }

    /// Creates a new node from its associated data.
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

    /// Counts the number of nodes in arena and returns it.
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if arena has no nodes, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns a reference to the node with the given id if in the arena.
    ///
    /// Returns `None` if not available.
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.index0())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    ///
    /// Returns `None` if not available.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.index0())
    }

    /// Returns an iterator of all nodes in the arena in storage-order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// [`is_removed()`]: struct.Node.html#method.is_removed
    pub fn iter(&self) -> Iter<Node<T>> {
        self.nodes.iter()
    }
}

#[cfg(feature = "par_iter")]
impl<T: Sync> Arena<T> {
    /// Returns an parallel iterator over the whole arena.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// [`is_removed()`]: struct.Node.html#method.is_removed
    pub fn par_iter(&self) -> rayon::slice::Iter<Node<T>> {
        self.nodes.par_iter()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
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

pub(crate) trait GetPairMut<T> {
    /// Get mutable references to two distinct nodes
    fn get_tuple_mut(&mut self, a: usize, b: usize) -> Option<(&mut T, &mut T)>;
}

impl<T> GetPairMut<T> for Vec<T> {
    fn get_tuple_mut(&mut self, a: usize, b: usize) -> Option<(&mut T, &mut T)> {
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
