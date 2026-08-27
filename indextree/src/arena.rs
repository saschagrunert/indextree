//! Arena structure and node storage.
//!
//! The [`Arena`] is the central owner of all tree nodes. Nodes are stored
//! contiguously in a single `Vec` and referenced by [`NodeId`](crate::NodeId).
//! Removed nodes are recycled through an internal free list.

#[cfg(not(feature = "std"))]
use alloc::vec::{self, Vec};

#[cfg(not(feature = "std"))]
use core::{
    mem,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice,
};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{
    mem,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice, vec,
};

use crate::{Node, NodeId, node::NodeData};

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// An `Arena` structure containing certain [`Node`]s.
pub struct Arena<T> {
    nodes: Vec<Node<T>>,
    first_free_slot: Option<usize>,
    last_free_slot: Option<usize>,
}

impl<T> Arena<T> {
    /// Creates a new empty `Arena`.
    #[must_use]
    pub const fn new() -> Arena<T> {
        Self {
            nodes: Vec::new(),
            first_free_slot: None,
            last_free_slot: None,
        }
    }

    /// Creates a new empty `Arena` with enough capacity to store `n` nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let arena: Arena<i32> = Arena::with_capacity(10);
    /// assert!(arena.capacity() >= 10);
    /// ```
    #[must_use]
    pub fn with_capacity(n: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(n),
            first_free_slot: None,
            last_free_slot: None,
        }
    }

    /// Returns the number of nodes the arena can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let arena: Arena<i32> = Arena::with_capacity(10);
    /// assert!(arena.capacity() >= 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Reserves capacity for `additional` more nodes to be inserted.
    ///
    /// The arena may reserve more space to avoid frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds isize::MAX bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena: Arena<i32> = Arena::new();
    /// arena.reserve(100);
    /// assert!(arena.capacity() >= 100);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
    }

    /// Retrieves the `NodeId` corresponding to a `Node` in the `Arena`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let node = arena.get(foo).unwrap();
    ///
    /// let node_id = arena.get_node_id(node).unwrap();
    /// assert_eq!(*arena[node_id].get(), "foo");
    /// ```
    pub fn get_node_id(&self, node: &Node<T>) -> Option<NodeId> {
        let nodes_range = self.nodes.as_ptr_range();
        let p = node as *const Node<T>;

        if !nodes_range.contains(&p) {
            return None;
        }

        let node_index = (p as usize - nodes_range.start as usize) / mem::size_of::<Node<T>>();
        let node_id = NonZeroUsize::new(node_index.wrapping_add(1))?;

        Some(NodeId::from_non_zero_usize(
            node_id,
            self.nodes[node_index].stamp,
        ))
    }

    /// Retrieves the `NodeId` corresponding to the `Node` at `index` in the `Arena`, if it exists.
    ///
    /// Note: We use 1 based indexing, so the first element is at `1` and not `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # use std::num::NonZeroUsize;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let node = arena.get(foo).unwrap();
    /// let index: NonZeroUsize = foo.into();
    ///
    /// let new_foo = arena.get_node_id_at(index).unwrap();
    /// assert_eq!(foo, new_foo);
    ///
    /// foo.remove(&mut arena);
    /// let new_foo = arena.get_node_id_at(index);
    /// assert!(new_foo.is_none(), "must be none if the node at the index doesn't exist");
    /// ```
    pub fn get_node_id_at(&self, index: NonZeroUsize) -> Option<NodeId> {
        let index0 = index.get() - 1; // we use 1 based indexing.
        self.nodes
            .get(index0)
            .filter(|n| !n.is_removed())
            .map(|node| NodeId::from_non_zero_usize(index, node.stamp))
    }

    /// Creates a new node from its associated data.
    ///
    /// Freed slots are reused when available. If a slot's internal stamp has
    /// been exhausted (after ~32K remove/reuse cycles), it is skipped and a
    /// fresh slot is appended instead.
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

    /// Returns the number of slots in the arena, including removed nodes.
    ///
    /// Removed nodes are still counted because they remain in the
    /// internal storage. Use [`iter()`] with [`Node::is_removed()`]
    /// to count only live nodes.
    ///
    /// [`iter()`]: Arena::iter
    /// [`Node::is_removed()`]: crate::Node::is_removed
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    /// assert_eq!(arena.count(), 2);
    /// assert_eq!(arena.len(), 2);
    ///
    /// foo.remove(&mut arena);
    /// // The removed node is still counted.
    /// assert_eq!(arena.count(), 2);
    /// assert_eq!(arena.len(), 2);
    /// ```
    #[deprecated(since = "4.9.0", note = "use len() instead")]
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of slots in the arena, including removed nodes.
    ///
    /// Removed nodes are still counted because they remain in the
    /// internal storage. Use [`iter()`] with [`Node::is_removed()`]
    /// to count only live nodes.
    ///
    /// [`iter()`]: Arena::iter
    /// [`Node::is_removed()`]: crate::Node::is_removed
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    /// assert_eq!(arena.len(), 2);
    ///
    /// foo.remove(&mut arena);
    /// // The removed node is still counted.
    /// assert_eq!(arena.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
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
        self.nodes.is_empty()
    }

    /// Returns a reference to the node with the given id if in the arena.
    ///
    /// Returns `None` if the index is out of bounds or the node's stamp
    /// does not match (i.e. the slot was removed and possibly reused).
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
    /// Stale `NodeId`s from removed nodes return `None`:
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// foo.remove(&mut arena);
    /// assert!(arena.get(foo).is_none());
    /// ```
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes
            .get(id.index0())
            .filter(|node| node.stamp == id.stamp())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    ///
    /// Returns `None` if the index is out of bounds or the node's stamp
    /// does not match.
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
    #[inline]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        let stamp = id.stamp();
        self.nodes
            .get_mut(id.index0())
            .filter(|node| node.stamp == stamp)
    }

    /// Returns a reference to the data of the node with the given id.
    ///
    /// Returns `None` if the id is out of bounds, the stamp doesn't match,
    /// or the node has been removed.
    ///
    /// This is a shorthand for `arena.get(id).map(|n| n.get())`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get_data(foo), Some(&"foo"));
    ///
    /// foo.remove(&mut arena);
    /// assert_eq!(arena.get_data(foo), None);
    /// ```
    #[inline]
    pub fn get_data(&self, id: NodeId) -> Option<&T> {
        self.get(id).map(|n| n.get())
    }

    /// Returns a mutable reference to the data of the node with the given id.
    ///
    /// Returns `None` if the id is out of bounds, the stamp doesn't match,
    /// or the node has been removed.
    ///
    /// This is a shorthand for `arena.get_mut(id).map(|n| n.get_mut())`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// *arena.get_data_mut(foo).unwrap() = "bar";
    /// assert_eq!(arena.get_data(foo), Some(&"bar"));
    /// ```
    #[inline]
    pub fn get_data_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.get_mut(id).map(|n| n.get_mut())
    }

    /// Returns an iterator of all nodes in the arena in storage-order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// To iterate over only live nodes by their [`NodeId`], use
    /// [`iter_node_ids()`](Arena::iter_node_ids) instead.
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
    /// [`is_removed()`]: Node::is_removed
    pub fn iter(&self) -> slice::Iter<'_, Node<T>> {
        self.nodes.iter()
    }

    /// Returns an iterator of [`NodeId`]s of all non-removed nodes in
    /// the arena in storage-order.
    ///
    /// Unlike [`iter()`], this skips removed nodes and yields `NodeId`s
    /// instead of `&Node<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// let baz = arena.new_node("baz");
    /// bar.remove(&mut arena);
    ///
    /// let ids: Vec<_> = arena.iter_node_ids().collect();
    /// assert_eq!(ids, vec![foo, baz]);
    /// ```
    ///
    /// [`iter()`]: Arena::iter
    pub fn iter_node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().enumerate().filter_map(|(i, node)| {
            if node.is_removed() {
                return None;
            }
            let index1 = NonZeroUsize::new(i.wrapping_add(1))?;
            Some(NodeId::from_non_zero_usize(index1, node.stamp))
        })
    }

    /// Returns a mutable iterator of all nodes in the arena in storage-order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// # Example
    ///
    /// ```
    /// # use indextree::Arena;
    /// let arena: &mut Arena<i64> = &mut Arena::new();
    /// let a = arena.new_node(1);
    /// let b = arena.new_node(2);
    /// assert!(a.checked_append(b, arena).is_ok());
    ///
    /// for node in arena.iter_mut() {
    ///     let data = node.get_mut();
    ///     *data = data.wrapping_add(4);
    /// }
    ///
    /// let node_refs = arena.iter().map(|i| i.get().clone()).collect::<Vec<_>>();
    /// assert_eq!(node_refs, vec![5, 6]);
    /// ```
    /// [`is_removed()`]: Node::is_removed
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Node<T>> {
        self.nodes.iter_mut()
    }

    /// Returns an iterator of [`NodeId`]s of all root nodes (nodes with
    /// no parent) that are not removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let a = arena.new_node("a");
    /// let b = arena.new_node("b");
    /// let c = arena.new_node("c");
    /// a.append(c, &mut arena);
    ///
    /// let roots: Vec<_> = arena.roots().collect();
    /// assert_eq!(roots, vec![a, b]);
    /// ```
    pub fn roots(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.iter_node_ids()
            .filter(|&id| self[id].parent().is_none())
    }

    /// Creates a new arena by applying a function to the data of every
    /// live node, preserving the tree structure.
    ///
    /// Removed nodes remain as removed slots in the new arena to keep
    /// node indices consistent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node(1);
    /// let child = root.append_value(2, &mut arena);
    ///
    /// let mapped: Arena<String> = arena.map(|x| x.to_string());
    /// assert_eq!(mapped.get_data(root), Some(&"1".to_string()));
    /// assert_eq!(mapped.get_data(child), Some(&"2".to_string()));
    /// assert_eq!(mapped[child].parent(), Some(root));
    /// ```
    pub fn map<U>(&self, mut f: impl FnMut(&T) -> U) -> Arena<U> {
        let nodes = self
            .nodes
            .iter()
            .map(|node| Node {
                parent: node.parent,
                previous_sibling: node.previous_sibling,
                next_sibling: node.next_sibling,
                first_child: node.first_child,
                last_child: node.last_child,
                stamp: node.stamp,
                data: match &node.data {
                    NodeData::Data(data) => NodeData::Data(f(data)),
                    NodeData::NextFree(next) => NodeData::NextFree(*next),
                },
            })
            .collect();
        Arena {
            nodes,
            first_free_slot: self.first_free_slot,
            last_free_slot: self.last_free_slot,
        }
    }

    /// Shrinks the internal storage to fit the current number of nodes.
    ///
    /// Calls [`Vec::shrink_to_fit`] on the underlying node storage.
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    /// Returns the number of live (non-removed) nodes in the arena.
    ///
    /// This is O(n) as it scans all slots. For the total slot count
    /// (including removed nodes), use [`len()`](Arena::len).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// assert_eq!(arena.live_count(), 2);
    ///
    /// foo.remove(&mut arena);
    /// assert_eq!(arena.live_count(), 1);
    /// assert_eq!(arena.len(), 2);
    /// ```
    pub fn live_count(&self) -> usize {
        self.nodes.iter().filter(|n| !n.is_removed()).count()
    }

    /// Clears all the nodes in the arena, but retains its allocated capacity.
    ///
    /// Note that this does not mark all nodes as removed, but completely
    /// removes them from the arena storage, thus invalidating all the node
    /// IDs that were previously created.
    ///
    /// After clearing, [`NodeId::is_removed`] returns `true` for any
    /// previously created ID (without panicking), and [`Arena::get`]
    /// returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// arena.clear();
    /// assert!(arena.is_empty());
    /// assert!(foo.is_removed(&arena));
    /// ```
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.first_free_slot = None;
        self.last_free_slot = None;
    }

    /// Returns a slice of the inner nodes collection.
    ///
    /// The slice contains all nodes in storage order, including removed
    /// nodes. Use [`Node::is_removed()`] to filter them out.
    ///
    /// [`Node::is_removed()`]: crate::Node::is_removed
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// arena.new_node("foo");
    /// arena.new_node("bar");
    /// assert_eq!(arena.as_slice().len(), 2);
    /// ```
    pub fn as_slice(&self) -> &[Node<T>] {
        self.nodes.as_slice()
    }

    /// Validates the internal consistency of the arena's tree structure.
    ///
    /// Returns `true` if all parent-child and sibling pointers are
    /// consistent, the free list is valid, and no cycles exist in
    /// sibling chains. This is primarily useful after deserialization
    /// to detect corrupted data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node(1);
    /// root.append_value(2, &mut arena);
    /// assert!(arena.validate());
    /// ```
    pub fn validate(&self) -> bool {
        let len = self.nodes.len();

        for (i, node) in self.nodes.iter().enumerate() {
            if node.is_removed() {
                continue;
            }

            let check_id =
                |id: NodeId| -> bool { id.index0() < len && !self.nodes[id.index0()].is_removed() };

            if let Some(parent) = node.parent {
                if !check_id(parent) {
                    return false;
                }
                let p = &self.nodes[parent.index0()];
                let mut found = false;
                let mut child = p.first_child;
                let mut steps = 0;
                while let Some(c) = child {
                    if c.index0() >= len {
                        return false;
                    }
                    if c.index0() == i {
                        found = true;
                        break;
                    }
                    child = self.nodes[c.index0()].next_sibling;
                    steps += 1;
                    if steps > len {
                        return false;
                    }
                }
                if !found {
                    return false;
                }
            }

            if let Some(prev) = node.previous_sibling {
                if !check_id(prev)
                    || self.nodes[prev.index0()].next_sibling.map(|n| n.index0()) != Some(i)
                {
                    return false;
                }
            }
            if let Some(next) = node.next_sibling {
                if !check_id(next)
                    || self.nodes[next.index0()]
                        .previous_sibling
                        .map(|n| n.index0())
                        != Some(i)
                {
                    return false;
                }
            }
            if let Some(first) = node.first_child {
                if !check_id(first)
                    || self.nodes[first.index0()].parent.map(|n| n.index0()) != Some(i)
                {
                    return false;
                }
            }
            if let Some(last) = node.last_child {
                if !check_id(last)
                    || self.nodes[last.index0()].parent.map(|n| n.index0()) != Some(i)
                {
                    return false;
                }
            }
            if node.first_child.is_some() != node.last_child.is_some() {
                return false;
            }
        }

        // Validate free list
        if self.first_free_slot.is_some() != self.last_free_slot.is_some() {
            return false;
        }
        let mut free_count = 0;
        let mut last_visited = None;
        let mut slot = self.first_free_slot;
        while let Some(idx) = slot {
            if idx >= len {
                return false;
            }
            let node = &self.nodes[idx];
            if !node.is_removed() {
                return false;
            }
            match node.data {
                NodeData::NextFree(next) => slot = next,
                _ => return false,
            }
            last_visited = Some(idx);
            free_count += 1;
            if free_count > len {
                return false;
            }
        }

        self.last_free_slot == last_visited
    }

    pub(crate) fn free_node(&mut self, id: NodeId) {
        let node = &mut self[id];
        if node.is_removed() {
            return;
        }
        node.data = NodeData::NextFree(None);
        node.stamp.mark_removed();
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
                unreachable!("A data node considered as a freed node");
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
    /// Returns a parallel iterator over the whole arena.
    ///
    /// Requires the `par_iter` feature. Uses [rayon](https://docs.rs/rayon)
    /// for data parallelism across all nodes in storage order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # use rayon::prelude::*;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node(1);
    /// root.append_value(2, &mut arena);
    /// root.append_value(3, &mut arena);
    ///
    /// let sum: i64 = arena.par_iter().map(|node| *node.get()).sum();
    /// assert_eq!(sum, 6);
    /// ```
    ///
    /// [`is_removed()`]: Node::is_removed
    pub fn par_iter(&self) -> rayon::slice::Iter<'_, Node<T>> {
        self.nodes.par_iter()
    }
}

#[cfg(feature = "par_iter")]
impl<T: Send> Arena<T> {
    /// Returns a mutable parallel iterator over the whole arena.
    ///
    /// Requires the `par_iter` feature. Uses [rayon](https://docs.rs/rayon)
    /// for data parallelism across all nodes in storage order.
    ///
    /// Note that this iterator returns also removed elements, which can be
    /// tested with the [`is_removed()`] method on the node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # use rayon::prelude::*;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node(1);
    /// root.append_value(2, &mut arena);
    /// root.append_value(3, &mut arena);
    ///
    /// arena.par_iter_mut().for_each(|node| {
    ///     if let Some(data) = node.try_get_mut() {
    ///         *data *= 10;
    ///     }
    /// });
    ///
    /// let sum: i64 = arena.par_iter().map(|node| *node.get()).sum();
    /// assert_eq!(sum, 60);
    /// ```
    ///
    /// [`is_removed()`]: Node::is_removed
    pub fn par_iter_mut(&mut self) -> rayon::slice::IterMut<'_, Node<T>> {
        self.nodes.par_iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = &'a Node<T>;
    type IntoIter = slice::Iter<'a, Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = &'a mut Node<T>;
    type IntoIter = slice::IterMut<'a, Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for Arena<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.new_node(item);
        }
    }
}

impl<T> core::iter::FromIterator<T> for Arena<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut arena = Arena::new();
        arena.extend(iter);
        arena
    }
}

/// Consumes the arena, returning an iterator over all nodes (including removed ones).
impl<T> IntoIterator for Arena<T> {
    type Item = Node<T>;
    type IntoIter = vec::IntoIter<Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
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

/// Index by [`NodeId`] for convenient `arena[id]` access.
///
/// Unlike [`Arena::get`], this does **not** validate the node's stamp,
/// so it may silently return data from a reused slot if the `NodeId`
/// is stale. For safe access, prefer [`Arena::get`] or [`Arena::get_mut`].
///
/// # Panics
///
/// Panics if `node` is out of bounds. Note that indexing does not validate
/// that the `NodeId` originated from this arena. Using an ID from a
/// different arena may silently access the wrong node or panic.
impl<T> Index<NodeId> for Arena<T> {
    type Output = Node<T>;

    #[inline]
    fn index(&self, node: NodeId) -> &Node<T> {
        &self.nodes[node.index0()]
    }
}

impl<T> IndexMut<NodeId> for Arena<T> {
    #[inline]
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

#[test]
fn conserve_capacity() {
    let mut arena = Arena::with_capacity(5);
    let cap = arena.capacity();
    assert!(cap >= 5);
    for i in 0..cap {
        arena.new_node(i);
    }
    arena.clear();
    assert!(arena.is_empty());
    let n1_id = arena.new_node(1);
    let n2_id = arena.new_node(2);
    let n3_id = arena.new_node(3);
    assert_eq!(n1_id.index0(), 0);
    assert_eq!(n2_id.index0(), 1);
    assert_eq!(n3_id.index0(), 2);
    assert_eq!(arena.len(), 3);
    assert_eq!(arena.capacity(), cap);
}

#[test]
fn stamp_no_cycle() {
    // Regression test for issue #95: stamps should never cycle back to
    // a previously used value after many reuse rounds.
    let mut arena = Arena::new();
    for _ in 0..=i16::MAX as u32 + 1 {
        let id = arena.new_node(42);
        assert!(!id.is_removed(&arena));
        id.remove(&mut arena);
        assert!(id.is_removed(&arena));
        let new_id = arena.new_node(42);
        assert!(!new_id.is_removed(&arena));
        assert!(id.is_removed(&arena));
        new_id.remove(&mut arena);
    }
}
