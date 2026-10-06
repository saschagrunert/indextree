//! Arena structure and node storage.
//!
//! The [`Arena`] is the central owner of all tree nodes. Nodes are stored
//! contiguously in a single `Vec` and referenced by [`NodeId`](crate::NodeId).
//! Removed nodes are recycled through an internal free list.

#[cfg(not(feature = "std"))]
use alloc::{
    vec,
    vec::{IntoIter, Vec},
};

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
    slice,
    vec::IntoIter,
};

use crate::{Node, NodeId, node::NodeData};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// An `Arena` structure containing certain [`Node`]s.
///
/// With the `serde` feature, deserialization fails if the arena does not
/// pass [`Arena::validate`].
pub struct Arena<T> {
    nodes: Vec<Node<T>>,
    first_free_slot: Option<usize>,
    last_free_slot: Option<usize>,
    /// Number of live (non-removed) nodes. Derived from `nodes`, so it is not
    /// serialized.
    #[cfg_attr(feature = "serde", serde(skip))]
    live: usize,
}

impl<T> Arena<T> {
    /// Creates a new empty `Arena`.
    #[must_use]
    pub const fn new() -> Arena<T> {
        Self {
            nodes: Vec::new(),
            first_free_slot: None,
            last_free_slot: None,
            live: 0,
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
            live: 0,
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
    /// Returns `None` if the node is not stored in this arena or was removed.
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
        let stamp = self.nodes[node_index].stamp;
        if stamp.is_removed() {
            return None;
        }

        Some(NodeId::from_non_zero_usize(node_id, stamp))
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
        self.live += 1;
        let next_index1 =
            NonZeroUsize::new(index.wrapping_add(1)).expect("Too many nodes in the arena");
        NodeId::from_non_zero_usize(next_index1, stamp)
    }

    /// Returns the number of slots in the arena, including removed nodes.
    ///
    /// Removed nodes are still counted because they remain in the
    /// internal storage. Use [`live_count()`](Arena::live_count) to count
    /// only live nodes.
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
    /// internal storage. Use [`live_count()`](Arena::live_count) to count
    /// only live nodes.
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

    /// Returns `true` if arena has no slots, `false` otherwise.
    ///
    /// Like [`len()`](Arena::len), this takes removed nodes into account, so
    /// an arena whose nodes have all been removed is not empty. Check
    /// [`live_count()`](Arena::live_count) for zero to test for live nodes.
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
            live: self.live,
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
    /// This is O(1). The count is maintained by the arena, so it can be off
    /// after replacing whole nodes through [`IndexMut`], [`Arena::iter_mut`]
    /// or `par_iter_mut`, e.g. by swapping nodes between arenas. For the
    /// total slot count
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
        self.live
    }

    /// Clears all the nodes in the arena, but retains its allocated capacity.
    ///
    /// Note that this does not mark all nodes as removed, but completely
    /// removes them from the arena storage, so all previously created node
    /// IDs must no longer be used.
    ///
    /// After clearing, [`NodeId::is_removed`] returns `true` for any
    /// previously created ID (without panicking), and [`Arena::get`]
    /// returns `None`, until new nodes are created. Since the slots and
    /// their stamps are discarded, new nodes reuse the indices and initial
    /// stamps of the cleared ones, so an old ID may then refer to a new node.
    /// Discard all IDs of the cleared arena, or remove the nodes with
    /// [`NodeId::remove_subtree`] instead to keep stale IDs detectable.
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
        self.live = 0;
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
    /// Returns `true` if the arena passes all of the following checks:
    ///
    /// - Every live node contains actual data (not a free-list entry).
    /// - Parent, previous/next sibling, and first/last child pointers
    ///   all refer to valid, non-removed nodes with matching stamps.
    /// - `first_child` and `last_child` are both set or both unset.
    /// - Sibling back-pointers are reciprocal (prev's next == self,
    ///   next's prev == self).
    /// - Every child in a parent's child chain points back to that
    ///   parent, and the chain ends at `last_child`.
    /// - Every node claiming a parent appears in that parent's child
    ///   chain.
    /// - No cycles exist in sibling chains (bounded by arena length).
    /// - The free list is well-formed: consistent first/last pointers,
    ///   all entries are removed nodes with `NextFree` data, and no
    ///   cycles.
    ///
    /// Deserialization (with the `serde` feature) already runs this check.
    /// It is useful to verify an arena after modifying nodes directly, e.g.
    /// through [`IndexMut`] or [`Arena::iter_mut`].
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
    #[must_use]
    pub fn validate(&self) -> bool {
        let len = self.nodes.len();

        let is_valid = |id: NodeId| -> bool {
            let idx = id.index0();
            idx < len && self.nodes[idx].stamp == id.stamp() && !self.nodes[idx].is_removed()
        };

        let mut in_child_chain = vec![false; len];

        for (i, node) in self.nodes.iter().enumerate() {
            if node.is_removed() {
                continue;
            }

            if !matches!(node.data, NodeData::Data(_)) {
                return false;
            }

            if let Some(parent) = node.parent {
                if !is_valid(parent) {
                    return false;
                }
            }

            if let Some(prev) = node.previous_sibling {
                if !is_valid(prev)
                    || self.nodes[prev.index0()].next_sibling.map(|n| n.index0()) != Some(i)
                {
                    return false;
                }
            }
            if let Some(next) = node.next_sibling {
                if !is_valid(next)
                    || self.nodes[next.index0()]
                        .previous_sibling
                        .map(|n| n.index0())
                        != Some(i)
                {
                    return false;
                }
            }

            if node.first_child.is_some() != node.last_child.is_some() {
                return false;
            }

            if let Some(first) = node.first_child {
                if !is_valid(first) {
                    return false;
                }
                let mut child = Some(first);
                let mut last_seen = first;
                let mut steps = 0;
                while let Some(c) = child {
                    let idx = c.index0();
                    if idx >= len {
                        return false;
                    }
                    let child_node = &self.nodes[idx];
                    if child_node.parent.map(|n| n.index0()) != Some(i) {
                        return false;
                    }
                    in_child_chain[idx] = true;
                    last_seen = c;
                    child = child_node.next_sibling;
                    steps += 1;
                    if steps > len {
                        return false;
                    }
                }
                if node.last_child.map(|n| n.index0()) != Some(last_seen.index0()) {
                    return false;
                }
            }
        }

        for (i, node) in self.nodes.iter().enumerate() {
            if !node.is_removed() && node.parent.is_some() && !in_child_chain[i] {
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
        let node = &mut self.nodes[id.index0()];
        if node.is_removed() {
            return;
        }
        // Saturating, because nodes can be swapped between arenas through
        // `IndexMut`, which this count cannot track.
        self.live = self.live.saturating_sub(1);
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

    /// Removes slots that can no longer be reused from the free list.
    ///
    /// Older versions kept slots in the free list one generation longer than
    /// [`NodeStamp::reuseable`](crate::id::NodeStamp) allows, so such slots
    /// can appear in deserialized arenas. Must only be called on an arena
    /// that passed [`Arena::validate`].
    #[cfg(feature = "serde")]
    fn unlink_exhausted_free_slots(&mut self) {
        let mut previous: Option<usize> = None;
        let mut slot = self.first_free_slot;
        while let Some(index) = slot {
            let NodeData::NextFree(next) = self.nodes[index].data else {
                unreachable!("validated free list entries are free nodes");
            };
            if self.nodes[index].stamp.reuseable() {
                previous = Some(index);
            } else {
                match previous {
                    Some(previous) => self.nodes[previous].data = NodeData::NextFree(next),
                    None => self.first_free_slot = next,
                }
                self.nodes[index].data = NodeData::NextFree(None);
            }
            slot = next;
        }
        self.last_free_slot = previous;
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
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        self.reserve(lower);
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
    type IntoIter = IntoIter<Node<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

// The live count is derived from the nodes, so it is not compared.
impl<T: PartialEq> PartialEq for Arena<T> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
            && self.first_free_slot == other.first_free_slot
            && self.last_free_slot == other.last_free_slot
    }
}

impl<T: Eq> Eq for Arena<T> {}

impl<T: core::fmt::Debug> core::fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The live count is derived from the nodes, so leave it out.
        f.debug_struct("Arena")
            .field("nodes", &self.nodes)
            .field("first_free_slot", &self.first_free_slot)
            .field("last_free_slot", &self.last_free_slot)
            .finish()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arena<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Same layout as the serialized `Arena`.
        #[derive(Deserialize)]
        #[serde(rename = "Arena")]
        struct ArenaData<T> {
            nodes: Vec<Node<T>>,
            first_free_slot: Option<usize>,
            last_free_slot: Option<usize>,
        }

        let data = ArenaData::deserialize(deserializer)?;
        let mut arena = Arena {
            live: data.nodes.iter().filter(|n| !n.is_removed()).count(),
            nodes: data.nodes,
            first_free_slot: data.first_free_slot,
            last_free_slot: data.last_free_slot,
        };
        if !arena.validate() {
            return Err(serde::de::Error::custom(
                "invalid arena: inconsistent node links or free list",
            ));
        }
        arena.unlink_exhausted_free_slots();
        Ok(arena)
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

/// Mutable index by [`NodeId`].
///
/// Like [`Index<NodeId>`], this does **not** validate the node's stamp.
/// For safe access, prefer [`Arena::get_mut`].
///
/// # Panics
///
/// Panics if `node` is out of bounds.
impl<T> IndexMut<NodeId> for Arena<T> {
    #[inline]
    fn index_mut(&mut self, node: NodeId) -> &mut Node<T> {
        &mut self.nodes[node.index0()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
