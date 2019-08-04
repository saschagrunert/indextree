//! Arena.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use core::{
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
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

use crate::{Node, NodeId};

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// An `Arena` structure containing certain [`Node`]s.
///
/// [`Node`]: struct.Node.html
pub struct Arena<T> {
    nodes: Vec<Node<T>>,
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    ///
    /// assert_eq!(arena[foo].data, "foo");
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    /// assert_eq!(arena.count(), 2);
    ///
    /// let _ = foo.remove(&mut arena)?;
    /// assert_eq!(arena.count(), 2);
    /// # Ok::<(), failure::Error>(())
    /// ```
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if arena has no nodes, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// assert!(arena.is_empty());
    ///
    /// let foo = arena.new_node("foo");
    /// assert!(!arena.is_empty());
    ///
    /// foo.remove(&mut arena)?;
    /// assert!(!arena.is_empty());
    /// # Ok::<(), failure::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns a reference to the node with the given id if in the arena.
    ///
    /// Returns `None` if not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| node.data), Some("foo"));
    ///
    /// let ghost = NodeId::new(42);
    /// assert!(arena.get(ghost).is_none());
    /// ```
    ///
    /// Note that this does not check whether the given node ID is created by
    /// the arena.
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| node.data), Some("foo"));
    ///
    /// let mut another_arena = Arena::new();
    /// let _ = another_arena.new_node("Another arena");
    /// assert_eq!(another_arena.get(foo).map(|node| node.data), Some("Another arena"));
    /// ```
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.index0())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    ///
    /// Returns `None` if not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| node.data), Some("foo"));
    ///
    /// let ghost = NodeId::new(42);
    /// assert!(arena.get_mut(ghost).is_none());
    ///
    /// arena.get_mut(foo).expect("The `foo` node exists").data = "FOO!";
    /// assert_eq!(arena.get(foo).map(|node| node.data), Some("FOO!"));
    /// ```
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.index0())
    }

    /// Returns an iterator of all nodes in the arena in storage-order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    ///
    /// let mut iter = arena.iter();
    /// assert_eq!(iter.next().map(|node| node.data), Some("foo"));
    /// assert_eq!(iter.next().map(|node| node.data), Some("bar"));
    /// assert_eq!(iter.next().map(|node| node.data), None);
    /// ```
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// bar.remove(&mut arena)?;
    ///
    /// let mut iter = arena.iter();
    /// assert_eq!(iter.next().map(|node| (node.data, node.is_removed())), Some(("foo", false)));
    /// assert_eq!(iter.next().map(|node| (node.data, node.is_removed())), Some(("bar", true)));
    /// assert_eq!(iter.next().map(|node| (node.data, node.is_removed())), None);
    /// # Ok::<(), failure::Error>(())
    /// ```
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
