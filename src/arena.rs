//! Arena.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use core::{
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};

use crate::{node::NodeData, Node, NodeId};

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// An `Arena` structure containing certain [`Node`]s.
///
/// [`Node`]: struct.Node.html
pub struct Arena<T> {
    nodes: Vec<Node<T>>,
    first_free_slot: Option<usize>,
    last_free_slot: Option<usize>,
}

impl<T> Arena<T> {
    /// Creates a new empty `Arena`.
    pub fn new() -> Arena<T> {
        Self::default()
    }

    /// Retrieves the `NodeId` correspoding to a `Node` in the `Arena`.
    /// Note that this method can only be used if the datatype of the `Node` implements
    /// the `PartialEq` trait.
    pub fn get_node_id(&self, node: &Node<T>) -> Option<NodeId>
    where
        T: PartialEq,
    {
        if let Some(node_id) = self.nodes.iter().position(|n| n.data == node.data) {
            NonZeroUsize::new(node_id.wrapping_add(1)).map(|node_id_non_zero| {
                NodeId::from_non_zero_usize(node_id_non_zero, self.nodes[node_id].stamp)
            })
        } else {
            None
        }
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
    /// assert_eq!(*arena[foo].get(), "foo");
    /// ```
    pub fn new_node(&mut self, data: T) -> NodeId {
        let (index, stamp) = if let Some(index) = self.pop_front_free_node() {
            let node = &mut self.nodes[index];
            node.reuse(data);
            (index, node.stamp)
        } else {
            let index = self.nodes.len();
            let node = Node::new(data);
            let stamp = node.stamp;
            self.nodes.push(node);
            (index, stamp)
        };
        let next_index1 =
            NonZeroUsize::new(index.wrapping_add(1)).expect("Too many nodes in the arena");
        NodeId::from_non_zero_usize(next_index1, stamp)
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
    /// foo.remove(&mut arena);
    /// assert_eq!(arena.count(), 2);
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
    /// foo.remove(&mut arena);
    /// assert!(!arena.is_empty());
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
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    /// ```
    ///
    /// Note that this does not check whether the given node ID is created by
    /// the arena.
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    ///
    /// let mut another_arena = Arena::new();
    /// let _ = another_arena.new_node("Another arena");
    /// assert_eq!(another_arena.get(foo).map(|node| *node.get()), Some("Another arena"));
    /// assert!(another_arena.get(bar).is_none());
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
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    ///
    /// *arena.get_mut(foo).expect("The `foo` node exists").get_mut() = "FOO!";
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("FOO!"));
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
    /// assert_eq!(iter.next().map(|node| *node.get()), Some("foo"));
    /// assert_eq!(iter.next().map(|node| *node.get()), Some("bar"));
    /// assert_eq!(iter.next().map(|node| *node.get()), None);
    /// ```
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// bar.remove(&mut arena);
    ///
    /// let mut iter = arena.iter();
    /// assert_eq!(iter.next().map(|node| (*node.get(), node.is_removed())), Some(("foo", false)));
    /// assert_eq!(iter.next().map_or(false, |node| node.is_removed()), true);
    /// assert_eq!(iter.next().map(|node| (*node.get(), node.is_removed())), None);
    /// ```
    ///
    /// [`is_removed()`]: struct.Node.html#method.is_removed
    pub fn iter(&self) -> impl Iterator<Item = &Node<T>> {
        self.nodes.iter()
    }

    pub(crate) fn free_node(&mut self, id: NodeId) {
        let node = &mut self[id];
        node.data = NodeData::NextFree(None);
        node.stamp.as_removed();
        let stamp = node.stamp;
        if stamp.reuseable() {
            if let Some(index) = self.last_free_slot {
                let new_last = id.index0();
                self.nodes[index].data = NodeData::NextFree(Some(new_last));
                self.last_free_slot = Some(new_last);
            } else {
                debug_assert!(self.first_free_slot.is_none());
                debug_assert!(self.last_free_slot.is_none());
                self.first_free_slot = Some(id.index0());
                self.last_free_slot = Some(id.index0());
            }
        }
    }

    fn pop_front_free_node(&mut self) -> Option<usize> {
        let first = self.first_free_slot.take();
        if let Some(index) = first {
            if let NodeData::NextFree(next_free) = self.nodes[index].data {
                self.first_free_slot = next_free;
            } else {
                unreachable!("A data node consider as a freed node");
            }
            if self.first_free_slot.is_none() {
                self.last_free_slot = None;
            }
        }

        first
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
    pub fn par_iter(&self) -> rayon::slice::Iter<'_, Node<T>> {
        self.nodes.par_iter()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            first_free_slot: None,
            last_free_slot: None,
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

#[test]
fn reuse_node() {
    let mut arena = Arena::new();
    let n1_id = arena.new_node("1");
    let n2_id = arena.new_node("2");
    let n3_id = arena.new_node("3");
    n1_id.remove(&mut arena);
    n2_id.remove(&mut arena);
    n3_id.remove(&mut arena);
    let n1_id = arena.new_node("1");
    let n2_id = arena.new_node("2");
    let n3_id = arena.new_node("3");
    assert_eq!(n1_id.index0(), 0);
    assert_eq!(n2_id.index0(), 1);
    assert_eq!(n3_id.index0(), 2);
    assert_eq!(arena.nodes.len(), 3);
}
