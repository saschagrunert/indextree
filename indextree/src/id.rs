//! Node identifier and tree manipulation methods.
//!
//! [`NodeId`] is a lightweight handle used to reference a [`Node`](crate::Node)
//! within an [`Arena`](crate::Arena). Most tree operations (append, remove,
//! traverse) are methods on `NodeId` that take an `&Arena` or `&mut Arena`.

#[cfg(not(feature = "std"))]
use core::{fmt, num::NonZeroUsize};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{fmt, num::NonZeroUsize};

use crate::{
    Ancestors, Arena, BreadthFirstTraversal, Children, Descendants, FollowingSiblings, Leaves,
    NodeError, PrecedingSiblings, Predecessors, ReverseTraverse, Traverse,
    debug_pretty_print::DebugPrettyPrint,
    relations::{insert_first_unchecked, insert_last_unchecked, insert_with_neighbors},
    siblings_range::SiblingsRange,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// A node identifier within a particular [`Arena`].
///
/// This ID is used to get [`Node`](crate::Node) references from an [`Arena`].
///
/// # Cross-arena safety
///
/// A `NodeId` does not carry a reference to the arena it was created in.
/// Using an ID from one arena to index into a different arena will either
/// panic (if the index is out of bounds) or silently access the wrong node.
/// It is the caller's responsibility to use each `NodeId` only with its
/// originating arena.
pub struct NodeId {
    /// One-based index.
    index1: NonZeroUsize,
    stamp: NodeStamp,
}

/// A stamp for node reuse, used to detect if the node a `NodeId` points to
/// is still the same node.
///
/// Uses the sign of an `i16` as a removed flag: non-negative values represent
/// live nodes, negative values represent removed nodes. Each remove/reuse
/// cycle increments the effective generation by 1.
///
/// After approximately 32,766 remove/reuse cycles on the same slot, the
/// stamp saturates and the slot becomes permanently unreusable. New nodes
/// will be appended to the end of the arena instead. In practice this is
/// unlikely to matter unless a single slot is recycled in a tight loop.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub(crate) struct NodeStamp(i16);

impl NodeStamp {
    /// Returns `true` if this stamp represents a removed node (negative
    /// value).
    #[inline]
    pub(crate) fn is_removed(self) -> bool {
        self.0.is_negative()
    }

    /// Marks this stamp as removed by negating and offsetting the value.
    ///
    /// The resulting negative value differs from the live stamp, allowing
    /// `NodeId::is_removed` to detect stale references via inequality.
    pub(crate) fn mark_removed(&mut self) {
        debug_assert!(!self.is_removed());
        self.0 = if self.0 < i16::MAX {
            -self.0 - 1
        } else {
            -self.0
        };
    }

    /// Returns `true` if the node slot can be recycled.
    ///
    /// A slot becomes permanently unreusable once its generation nears
    /// `i16::MIN`, preventing stamp value collisions after many cycles.
    pub(crate) fn reuseable(self) -> bool {
        debug_assert!(self.is_removed());
        self.0 > i16::MIN + 1
    }

    /// Recycles this stamp for a new node, incrementing the generation.
    ///
    /// Negates the value back to positive, producing a new generation
    /// that differs from all previous stamps for this slot.
    pub(crate) fn reuse(&mut self) -> Self {
        debug_assert!(self.reuseable());
        self.0 = -self.0;
        *self
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index1)
    }
}

impl From<NodeId> for NonZeroUsize {
    fn from(value: NodeId) -> NonZeroUsize {
        value.index1
    }
}

impl From<NodeId> for usize {
    fn from(value: NodeId) -> usize {
        value.index1.get()
    }
}

impl NodeId {
    /// Returns zero-based index.
    #[inline]
    pub(crate) fn index0(self) -> usize {
        // This is totally safe because `self.index1 >= 1` is guaranteed by
        // `NonZeroUsize` type.
        self.index1.get() - 1
    }

    /// Creates a new `NodeId` from the given one-based index.
    #[inline]
    pub(crate) fn from_non_zero_usize(index1: NonZeroUsize, stamp: NodeStamp) -> Self {
        NodeId { index1, stamp }
    }

    /// Returns the stamp associated with this node ID.
    #[inline]
    pub(crate) fn stamp(self) -> NodeStamp {
        self.stamp
    }

    /// Returns `true` if the node this ID points to has been removed or
    /// is no longer present in the arena.
    ///
    /// Unlike indexing with `arena[id]`, this method does not panic when
    /// the node ID is out of bounds (e.g. after [`Arena::clear`]).
    ///
    /// [`Arena::clear`]: crate::Arena::clear
    pub fn is_removed<T>(self, arena: &Arena<T>) -> bool {
        match arena.as_slice().get(self.index0()) {
            Some(node) => node.stamp != self.stamp,
            None => true,
        }
    }

    /// Returns the ID of the parent node, unless this node is the root of the
    /// tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// # n1.append(n1_1, &mut arena);
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    /// assert_eq!(n1.parent(&arena), None);
    /// assert_eq!(n1_1.parent(&arena), Some(n1));
    /// assert_eq!(n1_2.parent(&arena), Some(n1));
    /// assert_eq!(n1_3.parent(&arena), Some(n1));
    /// ```
    pub fn parent<T>(self, arena: &Arena<T>) -> Option<Self> {
        arena[self].parent()
    }

    /// Returns the ID of the first child of this node, unless it has no child.
    ///
    /// Shorthand for `arena[self].first_child()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    ///
    /// assert_eq!(n1.first_child(&arena), Some(n1_1));
    /// assert_eq!(n1_1.first_child(&arena), None);
    /// ```
    #[inline]
    pub fn first_child<T>(self, arena: &Arena<T>) -> Option<Self> {
        arena[self].first_child()
    }

    /// Returns the ID of the last child of this node, unless it has no child.
    ///
    /// Shorthand for `arena[self].last_child()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    /// let n1_2 = n1.append_value("1_2", &mut arena);
    ///
    /// assert_eq!(n1.last_child(&arena), Some(n1_2));
    /// assert_eq!(n1_1.last_child(&arena), None);
    /// ```
    #[inline]
    pub fn last_child<T>(self, arena: &Arena<T>) -> Option<Self> {
        arena[self].last_child()
    }

    /// Returns the ID of the next sibling of this node.
    ///
    /// Shorthand for `arena[self].next_sibling()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    /// let n1_2 = n1.append_value("1_2", &mut arena);
    ///
    /// assert_eq!(n1_1.next_sibling(&arena), Some(n1_2));
    /// assert_eq!(n1_2.next_sibling(&arena), None);
    /// ```
    #[inline]
    pub fn next_sibling<T>(self, arena: &Arena<T>) -> Option<Self> {
        arena[self].next_sibling()
    }

    /// Returns the ID of the previous sibling of this node.
    ///
    /// Shorthand for `arena[self].previous_sibling()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    /// let n1_2 = n1.append_value("1_2", &mut arena);
    ///
    /// assert_eq!(n1_2.previous_sibling(&arena), Some(n1_1));
    /// assert_eq!(n1_1.previous_sibling(&arena), None);
    /// ```
    #[inline]
    pub fn previous_sibling<T>(self, arena: &Arena<T>) -> Option<Self> {
        arena[self].previous_sibling()
    }

    /// Returns `true` if this node has at least one child.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    ///
    /// assert!(n1.has_children(&arena));
    /// assert!(!n1_1.has_children(&arena));
    /// ```
    #[inline]
    pub fn has_children<T>(self, arena: &Arena<T>) -> bool {
        arena[self].first_child().is_some()
    }

    /// Returns `true` if this node has no children.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    ///
    /// assert!(!n1.is_leaf(&arena));
    /// assert!(n1_1.is_leaf(&arena));
    /// ```
    #[inline]
    pub fn is_leaf<T>(self, arena: &Arena<T>) -> bool {
        !self.has_children(arena)
    }

    /// Returns `true` if this node has no parent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    ///
    /// assert!(n1.is_root(&arena));
    /// assert!(!n1_1.is_root(&arena));
    /// ```
    #[inline]
    pub fn is_root<T>(self, arena: &Arena<T>) -> bool {
        arena[self].parent().is_none()
    }

    /// Returns an iterator of IDs of this node and its ancestors.
    ///
    /// Use [`.skip(1)`][`skip`] or call `.next()` once on the iterator to skip
    /// the node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_1_1_1 = arena.new_node("1_1_1_1");
    /// # n1_1_1.append(n1_1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #3
    /// //     |-- 1_1                                          // #2
    /// //     |   `-- 1_1_1 *                                  // #1
    /// //     |       `-- 1_1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_1_1.ancestors(&arena);
    /// assert_eq!(iter.next(), Some(n1_1_1));                  // #1
    /// assert_eq!(iter.next(), Some(n1_1));                    // #2
    /// assert_eq!(iter.next(), Some(n1));                      // #3
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn ancestors<T>(self, arena: &Arena<T>) -> Ancestors<'_, T> {
        Ancestors::new(arena, self)
    }

    /// Returns an iterator of IDs of this node and its predecessors.
    ///
    /// Use [`.skip(1)`][`skip`] or call `.next()` once on the iterator to skip
    /// the node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_1_1_1 = arena.new_node("1_1_1_1");
    /// # n1_1_1.append(n1_1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #3
    /// //     |-- 1_1                                          // #2
    /// //     |   `-- 1_1_1 *                                  // #1
    /// //     |       `-- 1_1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_1_1.predecessors(&arena);
    /// assert_eq!(iter.next(), Some(n1_1_1));                  // #1
    /// assert_eq!(iter.next(), Some(n1_1));                    // #2
    /// assert_eq!(iter.next(), Some(n1));                      // #3
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_2_1 = arena.new_node("1_2_1");
    /// # n1_2.append(n1_2_1, &mut arena);
    /// # let n1_2_1_1 = arena.new_node("1_2_1_1");
    /// # n1_2_1.append(n1_2_1_1, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// # let n1_4 = arena.new_node("1_4");
    /// # n1.append(n1_4, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #4
    /// //     |-- 1_1                                          // #3
    /// //     |-- 1_2                                          // #2
    /// //     |   `-- 1_2_1 *                                  // #1
    /// //     |       `-- 1_2_1_1
    /// //     |-- 1_3
    /// //     `-- 1_4
    ///
    /// let mut iter = n1_2_1.predecessors(&arena);
    /// assert_eq!(iter.next(), Some(n1_2_1));                  // #1
    /// assert_eq!(iter.next(), Some(n1_2));                    // #2
    /// assert_eq!(iter.next(), Some(n1_1));                    // #3
    /// assert_eq!(iter.next(), Some(n1));                      // #4
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn predecessors<T>(self, arena: &Arena<T>) -> Predecessors<'_, T> {
        Predecessors::new(arena, self)
    }

    /// Returns an iterator of IDs of this node and the siblings before it.
    ///
    /// Use [`.skip(1)`][`skip`] or call `.next()` once on the iterator to skip
    /// the node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1                                          // #2
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2                                          // #1
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_2.preceding_siblings(&arena);
    /// assert_eq!(iter.next(), Some(n1_2));                    // #1
    /// assert_eq!(iter.next(), Some(n1_1));                    // #2
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn preceding_siblings<T>(self, arena: &Arena<T>) -> PrecedingSiblings<'_, T> {
        PrecedingSiblings::new(arena, self)
    }

    /// Returns an iterator of IDs of this node and the siblings after
    /// it.
    ///
    /// Use [`.skip(1)`][`skip`] or call `.next()` once on the iterator to skip
    /// the node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2                                          // #1
    /// //     `-- 1_3                                          // #2
    ///
    /// let mut iter = n1_2.following_siblings(&arena);
    /// assert_eq!(iter.next(), Some(n1_2));                    // #1
    /// assert_eq!(iter.next(), Some(n1_3));                    // #2
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn following_siblings<T>(self, arena: &Arena<T>) -> FollowingSiblings<'_, T> {
        FollowingSiblings::new(arena, self)
    }

    /// Returns an iterator of IDs of this node’s children.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1                                          // #1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2                                          // #2
    /// //     `-- 1_3                                          // #3
    ///
    /// let mut iter = n1.children(&arena);
    /// assert_eq!(iter.next(), Some(n1_1));                    // #1
    /// assert_eq!(iter.next(), Some(n1_2));                    // #2
    /// assert_eq!(iter.next(), Some(n1_3));                    // #3
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn children<T>(self, arena: &Arena<T>) -> Children<'_, T> {
        Children::new(arena, self)
    }

    /// Returns the number of children of this node.
    ///
    /// This traverses the sibling chain and is O(n) in the number of
    /// children. If you only need to check whether a node has children,
    /// prefer checking `first_child()` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    /// assert_eq!(n1.child_count(&arena), 3);
    /// assert_eq!(n1_1.child_count(&arena), 0);
    /// ```
    pub fn child_count<T>(self, arena: &Arena<T>) -> usize {
        self.children(arena).count()
    }

    /// Returns the depth (level) of this node in the tree.
    ///
    /// A root node (no parent) has depth 0, its children have depth 1, etc.
    /// This traverses ancestors and is O(depth).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let child = root.append_value("child", &mut arena);
    /// let grandchild = child.append_value("grandchild", &mut arena);
    ///
    /// assert_eq!(root.depth(&arena), 0);
    /// assert_eq!(child.depth(&arena), 1);
    /// assert_eq!(grandchild.depth(&arena), 2);
    /// ```
    pub fn depth<T>(self, arena: &Arena<T>) -> usize {
        self.ancestors(arena).count() - 1
    }

    /// Returns the nth child of this node (zero-indexed).
    ///
    /// Returns `None` if the node has fewer than `n + 1` children.
    /// This is O(n) as it walks the sibling chain.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let c0 = root.append_value("c0", &mut arena);
    /// let c1 = root.append_value("c1", &mut arena);
    /// let c2 = root.append_value("c2", &mut arena);
    ///
    /// assert_eq!(root.nth_child(0, &arena), Some(c0));
    /// assert_eq!(root.nth_child(1, &arena), Some(c1));
    /// assert_eq!(root.nth_child(2, &arena), Some(c2));
    /// assert_eq!(root.nth_child(3, &arena), None);
    /// ```
    pub fn nth_child<T>(self, n: usize, arena: &Arena<T>) -> Option<NodeId> {
        self.children(arena).nth(n)
    }

    /// Returns `true` if this node is an ancestor of `other`.
    ///
    /// A node is not considered an ancestor of itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let child = root.append_value("child", &mut arena);
    /// let grandchild = child.append_value("grandchild", &mut arena);
    ///
    /// assert!(root.is_ancestor_of(child, &arena));
    /// assert!(root.is_ancestor_of(grandchild, &arena));
    /// assert!(!child.is_ancestor_of(root, &arena));
    /// assert!(!root.is_ancestor_of(root, &arena));
    /// ```
    pub fn is_ancestor_of<T>(self, other: NodeId, arena: &Arena<T>) -> bool {
        other.ancestors(arena).skip(1).any(|a| a == self)
    }

    /// Returns `true` if this node is a descendant of `other`.
    ///
    /// A node is not considered a descendant of itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let child = root.append_value("child", &mut arena);
    /// let grandchild = child.append_value("grandchild", &mut arena);
    ///
    /// assert!(grandchild.is_descendant_of(root, &arena));
    /// assert!(child.is_descendant_of(root, &arena));
    /// assert!(!root.is_descendant_of(child, &arena));
    /// assert!(!root.is_descendant_of(root, &arena));
    /// ```
    pub fn is_descendant_of<T>(self, other: NodeId, arena: &Arena<T>) -> bool {
        other.is_ancestor_of(self, arena)
    }

    /// An iterator of the IDs of a given node and its descendants, as a pre-order depth-first search where children are visited in insertion order.
    ///
    /// i.e. node -> first child -> second child
    ///
    /// Parent nodes appear before the descendants.
    /// Use [`.skip(1)`][`skip`] or call `.next()` once on the iterator to skip
    /// the node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_1_1_1 = arena.new_node("1_1_1_1");
    /// # n1_1_1.append(n1_1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #1
    /// //     |-- 1_1                                          // #2
    /// //     |   `-- 1_1_1                                    // #3
    /// //     |       `-- 1_1_1_1                              // #4
    /// //     |-- 1_2                                          // #5
    /// //     `-- 1_3                                          // #6
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));                      // #1
    /// assert_eq!(iter.next(), Some(n1_1));                    // #2
    /// assert_eq!(iter.next(), Some(n1_1_1));                  // #3
    /// assert_eq!(iter.next(), Some(n1_1_1_1));                // #4
    /// assert_eq!(iter.next(), Some(n1_2));                    // #5
    /// assert_eq!(iter.next(), Some(n1_3));                    // #6
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn descendants<T>(self, arena: &Arena<T>) -> Descendants<'_, T> {
        Descendants::new(arena, self)
    }

    /// Returns an iterator over the leaf nodes (nodes with no children)
    /// of this node's subtree in pre-order depth-first order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let a = root.append_value("a", &mut arena);
    /// let b = a.append_value("b", &mut arena);
    /// let c = root.append_value("c", &mut arena);
    ///
    /// let leaves: Vec<_> = root.leaves(&arena).collect();
    /// assert_eq!(leaves, vec![b, c]);
    /// ```
    pub fn leaves<T>(self, arena: &Arena<T>) -> Leaves<'_, T> {
        Leaves::new(arena, self)
    }

    /// Returns an iterator that yields nodes in breadth-first (level-order)
    /// order, starting from this node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node(1);
    /// let a = root.append_value(2, &mut arena);
    /// let b = root.append_value(3, &mut arena);
    /// let c = a.append_value(4, &mut arena);
    ///
    /// let bfs: Vec<_> = root.breadth_first(&arena)
    ///     .map(|id| *arena[id].get())
    ///     .collect();
    /// assert_eq!(bfs, vec![1, 2, 3, 4]);
    /// ```
    pub fn breadth_first<T>(self, arena: &Arena<T>) -> BreadthFirstTraversal<'_, T> {
        BreadthFirstTraversal::new(arena, self)
    }

    /// Returns the number of descendants of this node, including itself.
    ///
    /// This is O(n) in the size of the subtree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let a = root.append_value("a", &mut arena);
    /// a.append_value("b", &mut arena);
    /// root.append_value("c", &mut arena);
    ///
    /// assert_eq!(root.descendant_count(&arena), 4);
    /// assert_eq!(a.descendant_count(&arena), 2);
    /// ```
    #[must_use]
    pub fn descendant_count<T>(self, arena: &Arena<T>) -> usize {
        self.descendants(arena).count()
    }

    /// An iterator of the "sides" of a node visited during a depth-first pre-order traversal,
    /// where node sides are visited start to end and children are visited in insertion order.
    ///
    /// i.e. node.start -> first child -> second child -> node.end
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeEdge};
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #1, #10
    /// //     |-- 1_1                                          // #2, #5
    /// //     |   `-- 1_1_1                                    // #3, #4
    /// //     |-- 1_2                                          // #6, #7
    /// //     `-- 1_3                                          // #8, #9
    ///
    /// let mut iter = n1.traverse(&arena);
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1)));     // #1
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1)));   // #2
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1_1))); // #3
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1_1)));   // #4
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1)));     // #5
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_2)));   // #6
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_2)));     // #7
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_3)));   // #8
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_3)));     // #9
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1)));       // #10
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn traverse<T>(self, arena: &Arena<T>) -> Traverse<'_, T> {
        Traverse::new(arena, self)
    }

    /// An iterator of the "sides" of a node visited during a depth-first pre-order traversal,
    /// where nodes are visited end to start and children are visited in reverse insertion order.
    ///
    /// i.e. node.end -> second child -> first child -> node.start
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeEdge};
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #1, #10
    /// //     |-- 1_1                                          // #6, #9
    /// //     |   `-- 1_1_1                                    // #7, #8
    /// //     |-- 1_2                                          // #4, #5
    /// //     `-- 1_3                                          // #2, #3
    ///
    /// let mut iter = n1.reverse_traverse(&arena);
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1)));       // #1
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_3)));     // #2
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_3)));   // #3
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_2)));     // #4
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_2)));   // #5
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1)));     // #6
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1_1)));   // #7
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1_1))); // #8
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1)));   // #9
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1)));     // #10
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// ```
    /// # use indextree::{Arena, NodeEdge};
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1                                                // #1, #10
    /// //     |-- 1_1                                          // #6, #9
    /// //     |   `-- 1_1_1                                    // #7, #8
    /// //     |-- 1_2                                          // #4, #5
    /// //     `-- 1_3                                          // #2, #3
    /// let traverse = n1.traverse(&arena).collect::<Vec<_>>();
    /// let mut reverse = n1.reverse_traverse(&arena).collect::<Vec<_>>();
    /// reverse.reverse();
    /// assert_eq!(traverse, reverse);
    /// ```
    pub fn reverse_traverse<T>(self, arena: &Arena<T>) -> ReverseTraverse<'_, T> {
        ReverseTraverse::new(arena, self)
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    ///
    /// # Failures
    ///
    /// Returns [`NodeError::Removed`] if the node has been removed or the
    /// ID is stale.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// let child = root.append_value("child", &mut arena);
    /// assert!(child.checked_detach(&mut arena).is_ok());
    /// assert!(child.parent(&arena).is_none());
    /// ```
    pub fn checked_detach<T>(self, arena: &mut Arena<T>) -> Result<(), NodeError> {
        if self.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        self.detach(arena);
        Ok(())
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    ///
    /// # Panics
    ///
    /// Panics if the node ID is out of bounds (e.g. after
    /// [`Arena::clear`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeEdge};
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1");
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- (implicit)
    /// //     `-- 1
    /// //         |-- 1_1
    /// //         |   `-- 1_1_1
    /// //         |-- 1_2 *
    /// //         `-- 1_3
    ///
    /// n1_2.detach(&mut arena);
    /// // arena
    /// // |-- (implicit)
    /// // |   `-- 1
    /// // |       |-- 1_1
    /// // |       |   `-- 1_1_1
    /// // |       `-- 1_3
    /// // `-- (implicit)
    /// //     `-- 1_2 *
    ///
    /// assert!(arena[n1_2].parent().is_none());
    /// assert!(arena[n1_2].previous_sibling().is_none());
    /// assert!(arena[n1_2].next_sibling().is_none());
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn detach<T>(self, arena: &mut Arena<T>) {
        let range = SiblingsRange::new(self, self).detach_from_siblings(arena);
        range
            .rewrite_parents(arena, None)
            .expect("Should never happen: `None` as parent is always valid");

        // Ensure the node is surely detached.
        debug_assert!(
            arena[self].is_detached(),
            "The node should be successfully detached"
        );
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the given new child is `self`, or
    /// * the given new child is an ancestor of `self`, or
    /// * the current node or the given new child was already [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = arena.new_node("1_1");
    /// n1.append(n1_1, &mut arena);
    /// let n1_2 = arena.new_node("1_2");
    /// n1.append(n1_2, &mut arena);
    /// let n1_3 = arena.new_node("1_3");
    /// n1.append(n1_3, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn append<T>(self, new_child: NodeId, arena: &mut Arena<T>) {
        self.checked_append(new_child, arena)
            .expect("Preconditions not met: invalid argument");
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Failures
    ///
    /// * Returns [`NodeError::AppendSelf`] error if the given new child is
    ///   `self`.
    /// * Returns [`NodeError::AppendAncestor`] error if the given new child is
    ///   an ancestor of `self`.
    /// * Returns [`NodeError::Removed`] error if the given new child or `self`
    ///   is [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// assert!(n1.checked_append(n1, &mut arena).is_err());
    ///
    /// let n1_1 = arena.new_node("1_1");
    /// assert!(n1.checked_append(n1_1, &mut arena).is_ok());
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn checked_append<T>(
        self,
        new_child: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::AppendSelf);
        }
        if self.is_removed(arena) || new_child.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        if self.ancestors(arena).any(|ancestor| new_child == ancestor) {
            return Err(NodeError::AppendAncestor);
        }
        new_child.detach(arena);
        insert_with_neighbors(arena, new_child, Some(self), arena[self].last_child, None)
            .expect("Should never fail: `new_child` is not `self` and they are not removed");

        Ok(())
    }

    /// Creates and appends a new node (from its associated data) as the last child.
    /// This method is a fast path for the common case of appending a new node. It is quicker than [`append`].
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
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    /// let n1_1_1 = n1_1.append_value("1_1_1", &mut arena);
    /// let n1_1_2 = n1_1.append_value("1_1_2", &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     `-- 1_1
    /// //         |-- 1_1_1
    /// //         `-- 1_1_2
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_1_1));
    /// assert_eq!(iter.next(), Some(n1_1_2));
    /// assert_eq!(iter.next(), None);
    /// ```
    /// [`append`]: NodeId::append
    pub fn append_value<T>(self, value: T, arena: &mut Arena<T>) -> NodeId {
        let new_child = arena.new_node(value);
        self.append_new_node_unchecked(new_child, arena);

        new_child
    }

    /// Appends a new child to this node, after all existing children (if any).
    /// This method is a fast path for the common case of appending a new node.
    /// `new_child` requirements:
    /// 1. Must be detached. No parents or siblings.
    /// 2. Has not been [`remove`]d.
    /// 3. `append_new_node()` was not called on itself
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// assert!(n1.checked_append(n1, &mut arena).is_err());
    ///
    /// let n1_1 = arena.new_node("1_1");
    /// assert!(n1.checked_append(n1_1, &mut arena).is_ok());
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    fn append_new_node_unchecked<T>(self, new_child: NodeId, arena: &mut Arena<T>) {
        insert_last_unchecked(arena, new_child, self);
    }

    /// Creates and prepends a new node (from its associated data) as the first child.
    /// This method is a fast path for the common case of prepending a new node. It is quicker than [`prepend`].
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
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.prepend_value("1_1", &mut arena);
    /// let n1_2 = n1.prepend_value("1_2", &mut arena);
    /// let n1_3 = n1.prepend_value("1_3", &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_3
    /// //     |-- 1_2
    /// //     `-- 1_1
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), None);
    /// ```
    /// [`prepend`]: NodeId::prepend
    pub fn prepend_value<T>(self, value: T, arena: &mut Arena<T>) -> NodeId {
        let new_child = arena.new_node(value);
        insert_first_unchecked(arena, new_child, self);
        new_child
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the given new child is `self`, or
    /// * the given new child is an ancestor of `self`, or
    /// * the current node or the given new child was already [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = arena.new_node("1_1");
    /// n1.prepend(n1_1, &mut arena);
    /// let n1_2 = arena.new_node("1_2");
    /// n1.prepend(n1_2, &mut arena);
    /// let n1_3 = arena.new_node("1_3");
    /// n1.prepend(n1_3, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_3
    /// //     |-- 1_2
    /// //     `-- 1_1
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn prepend<T>(self, new_child: NodeId, arena: &mut Arena<T>) {
        self.checked_prepend(new_child, arena)
            .expect("Preconditions not met: invalid argument");
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Failures
    ///
    /// * Returns [`NodeError::PrependSelf`] error if the given new child is
    ///   `self`.
    /// * Returns [`NodeError::PrependAncestor`] error if the given new child is
    ///   an ancestor of `self`.
    /// * Returns [`NodeError::Removed`] error if the given new child or `self`
    ///   is [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// assert!(n1.checked_prepend(n1, &mut arena).is_err());
    ///
    /// let n1_1 = arena.new_node("1_1");
    /// assert!(n1.checked_prepend(n1_1, &mut arena).is_ok());
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn checked_prepend<T>(
        self,
        new_child: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::PrependSelf);
        }
        if self.is_removed(arena) || new_child.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        if self.ancestors(arena).any(|ancestor| new_child == ancestor) {
            return Err(NodeError::PrependAncestor);
        }
        new_child.detach(arena);
        insert_with_neighbors(arena, new_child, Some(self), None, arena[self].first_child)
            .expect("Should never fail: `new_child` is not `self` and they are not removed");

        Ok(())
    }

    /// Inserts a new sibling after this node.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the given new sibling is `self`, or
    /// * the current node or the given new sibling was already [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1 *
    /// //     `-- 1_2
    ///
    /// let n1_3 = arena.new_node("1_3");
    /// n1_1.insert_after(n1_3, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_3 *
    /// //     `-- 1_2
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn insert_after<T>(self, new_sibling: NodeId, arena: &mut Arena<T>) {
        self.checked_insert_after(new_sibling, arena)
            .expect("Preconditions not met: invalid argument");
    }

    /// Creates and inserts a new sibling node after this node.
    ///
    /// A convenience shorthand for creating a node via [`Arena::new_node`]
    /// and inserting it via [`insert_after`].
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the arena already has `usize::max_value()` nodes, or
    /// * `self` was already [`remove`]d.
    ///
    /// [`remove`]: NodeId::remove
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    /// let n1_3 = n1.append_value("1_3", &mut arena);
    /// let n1_2 = n1_1.insert_after_value("1_2", &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.children(&arena);
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`insert_after`]: NodeId::insert_after
    pub fn insert_after_value<T>(self, value: T, arena: &mut Arena<T>) -> NodeId {
        let new_sibling = arena.new_node(value);
        self.insert_after(new_sibling, arena);
        new_sibling
    }

    /// Inserts a new sibling after this node.
    ///
    /// # Failures
    ///
    /// * Returns [`NodeError::InsertAfterSelf`] error if the given new sibling
    ///   is `self`.
    /// * Returns [`NodeError::Removed`] error if the given new sibling or
    ///   `self` is [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// assert!(n1.checked_insert_after(n1, &mut arena).is_err());
    ///
    /// let n2 = arena.new_node("2");
    /// assert!(n1.checked_insert_after(n2, &mut arena).is_ok());
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn checked_insert_after<T>(
        self,
        new_sibling: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_sibling == self {
            return Err(NodeError::InsertAfterSelf);
        }
        if self.is_removed(arena) || new_sibling.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        new_sibling.detach(arena);
        let (next_sibling, parent) = {
            let current = &arena[self];
            (current.next_sibling, current.parent)
        };
        insert_with_neighbors(arena, new_sibling, parent, Some(self), next_sibling)
            .expect("Should never fail: `new_sibling` is not `self` and they are not removed");

        Ok(())
    }

    /// Inserts a new sibling before this node.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the given new sibling is `self`, or
    /// * the current node or the given new sibling was already [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = arena.new_node("1_1");
    /// n1.append(n1_1, &mut arena);
    /// let n1_2 = arena.new_node("1_2");
    /// n1.append(n1_2, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     `-- 1_2 *
    ///
    /// let n1_3 = arena.new_node("1_3");
    /// n1_2.insert_before(n1_3, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_3 *
    /// //     `-- 1_2
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn insert_before<T>(self, new_sibling: NodeId, arena: &mut Arena<T>) {
        self.checked_insert_before(new_sibling, arena)
            .expect("Preconditions not met: invalid argument");
    }

    /// Creates and inserts a new sibling node before this node.
    ///
    /// A convenience shorthand for creating a node via [`Arena::new_node`]
    /// and inserting it via [`insert_before`].
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    /// * the arena already has `usize::max_value()` nodes, or
    /// * `self` was already [`remove`]d.
    ///
    /// [`remove`]: NodeId::remove
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n1_1 = n1.append_value("1_1", &mut arena);
    /// let n1_3 = n1.append_value("1_3", &mut arena);
    /// let n1_2 = n1_3.insert_before_value("1_2", &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.children(&arena);
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`insert_before`]: NodeId::insert_before
    pub fn insert_before_value<T>(self, value: T, arena: &mut Arena<T>) -> NodeId {
        let new_sibling = arena.new_node(value);
        self.insert_before(new_sibling, arena);
        new_sibling
    }

    /// Inserts a new sibling before this node.
    ///
    /// # Failures
    ///
    /// * Returns [`NodeError::InsertBeforeSelf`] error if the given new sibling
    ///   is `self`.
    /// * Returns [`NodeError::Removed`] error if the given new sibling or
    ///   `self` is [`remove`]d.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// assert!(n1.checked_insert_before(n1, &mut arena).is_err());
    ///
    /// let n2 = arena.new_node("2");
    /// assert!(n1.checked_insert_before(n2, &mut arena).is_ok());
    /// ```
    ///
    /// [`remove`]: NodeId::remove
    pub fn checked_insert_before<T>(
        self,
        new_sibling: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_sibling == self {
            return Err(NodeError::InsertBeforeSelf);
        }
        if self.is_removed(arena) || new_sibling.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        new_sibling.detach(arena);
        let (previous_sibling, parent) = {
            let current = &arena[self];
            (current.previous_sibling, current.parent)
        };
        insert_with_neighbors(arena, new_sibling, parent, previous_sibling, Some(self))
            .expect("Should never fail: `new_sibling` is not `self` and they are not removed");

        Ok(())
    }

    /// Removes a node from the arena, returning an error on failure.
    ///
    /// Children of the removed node will be inserted in place of the
    /// removed node.
    ///
    /// # Failures
    ///
    /// Returns [`NodeError::Removed`] if the node has been removed or the
    /// ID is stale.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeError};
    /// let mut arena = Arena::new();
    /// let n = arena.new_node("x");
    /// assert!(n.checked_remove(&mut arena).is_ok());
    /// assert!(matches!(n.checked_remove(&mut arena), Err(NodeError::Removed)));
    /// ```
    pub fn checked_remove<T>(self, arena: &mut Arena<T>) -> Result<(), NodeError> {
        if self.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        self.remove(arena);
        Ok(())
    }

    /// Removes a node from the arena.
    ///
    /// Children of the removed node will be inserted to the place where the
    /// removed node was.
    ///
    /// Please note that the node will not be removed from the internal arena
    /// storage, but marked as `removed`. Traversing the arena returns a
    /// plain iterator and contains removed elements too.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`](crate::Node::is_removed).
    ///
    /// # Panics
    ///
    /// Panics if the node ID is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_2_1 = arena.new_node("1_2_1");
    /// # n1_2.append(n1_2_1, &mut arena);
    /// # let n1_2_2 = arena.new_node("1_2_2");
    /// # n1_2.append(n1_2_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2 *
    /// //     |   |-- 1_2_1
    /// //     |   `-- 1_2_2
    /// //     `-- 1_3
    ///
    /// n1_2.remove(&mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2_1
    /// //     |-- 1_2_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2_1));
    /// assert_eq!(iter.next(), Some(n1_2_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    pub fn remove<T>(self, arena: &mut Arena<T>) {
        debug_assert_triangle_nodes!(
            arena,
            arena[self].parent,
            arena[self].previous_sibling,
            Some(self)
        );
        debug_assert_triangle_nodes!(
            arena,
            arena[self].parent,
            Some(self),
            arena[self].next_sibling
        );
        debug_assert_triangle_nodes!(arena, Some(self), None, arena[self].first_child);
        debug_assert_triangle_nodes!(arena, Some(self), arena[self].last_child, None);

        // Retrieve needed values.
        let (parent, previous_sibling, next_sibling, first_child, last_child) = {
            let node = &arena[self];
            (
                node.parent,
                node.previous_sibling,
                node.next_sibling,
                node.first_child,
                node.last_child,
            )
        };

        assert_eq!(first_child.is_some(), last_child.is_some());
        self.detach(arena);
        if let (Some(first_child), Some(last_child)) = (first_child, last_child) {
            let range = SiblingsRange::new(first_child, last_child).detach_from_siblings(arena);
            range
                .transplant(arena, parent, previous_sibling, next_sibling)
                .expect("Should never fail: neighbors and children must be consistent");
        }
        arena.free_node(self);
        debug_assert!(arena[self].is_detached());
    }

    /// Removes a node and its descendants from the arena, returning an
    /// error on failure.
    ///
    /// # Failures
    ///
    /// Returns [`NodeError::Removed`] if the node has been removed or the
    /// ID is stale.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeError};
    /// let mut arena = Arena::new();
    /// let n = arena.new_node("x");
    /// n.append_value("child", &mut arena);
    /// assert!(n.checked_remove_subtree(&mut arena).is_ok());
    /// assert!(matches!(n.checked_remove_subtree(&mut arena), Err(NodeError::Removed)));
    /// ```
    pub fn checked_remove_subtree<T>(self, arena: &mut Arena<T>) -> Result<(), NodeError> {
        if self.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        self.remove_subtree(arena);
        Ok(())
    }

    /// Removes a node and its descendants from the arena.
    ///
    /// # Panics
    ///
    /// Panics if the node ID is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_2_1 = arena.new_node("1_2_1");
    /// # n1_2.append(n1_2_1, &mut arena);
    /// # let n1_2_2 = arena.new_node("1_2_2");
    /// # n1_2.append(n1_2_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2 *
    /// //     |   |-- 1_2_1
    /// //     |   `-- 1_2_2
    /// //     `-- 1_3
    ///
    /// n1_2.remove_subtree(&mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    pub fn remove_subtree<T>(self, arena: &mut Arena<T>) {
        self.detach(arena);

        let mut cursor = Some(self);
        while let Some(id) = cursor {
            let node = &arena[id];
            let first_child = node.first_child;
            let next_sibling = node.next_sibling;
            let parent = node.parent;
            arena.free_node(id);
            cursor = first_child.or(next_sibling).or_else(|| {
                let mut ancestor = parent;
                while let Some(a) = ancestor {
                    let ancestor_node = &arena[a];
                    if let Some(sib) = ancestor_node.next_sibling {
                        return Some(sib);
                    }
                    ancestor = ancestor_node.parent;
                }
                None
            });
        }
    }

    /// Detaches all children of this node, returning an error on failure.
    ///
    /// # Failures
    ///
    /// Returns [`NodeError::Removed`] if the node has been removed or the
    /// ID is stale.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// root.append_value("c1", &mut arena);
    /// assert!(root.checked_detach_children(&mut arena).is_ok());
    /// assert_eq!(root.children(&arena).count(), 0);
    /// ```
    pub fn checked_detach_children<T>(self, arena: &mut Arena<T>) -> Result<(), NodeError> {
        if self.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        self.detach_children(arena);
        Ok(())
    }

    /// Detaches all children of this node, leaving them as independent
    /// toplevel nodes while keeping the node itself in its current position.
    ///
    /// The children retain their own subtrees and sibling relationships
    /// with each other are removed.
    ///
    /// # Panics
    ///
    /// Panics if the node ID is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_2_1 = arena.new_node("1_2_1");
    /// # n1_2.append(n1_2_1, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     |   `-- 1_2_1
    /// //     `-- 1_3
    ///
    /// n1.detach_children(&mut arena);
    ///
    /// // arena (all former children are now independent toplevel nodes)
    /// // |-- 1
    /// // |-- 1_1
    /// // |-- 1_2
    /// // |   `-- 1_2_1
    /// // `-- 1_3
    ///
    /// assert_eq!(n1.children(&arena).count(), 0);
    /// assert!(!arena[n1_1].is_removed());
    /// assert!(arena[n1_1].parent().is_none());
    /// // 1_2's subtree is preserved
    /// assert_eq!(arena[n1_2_1].parent(), Some(n1_2));
    /// ```
    pub fn detach_children<T>(self, arena: &mut Arena<T>) {
        let first = arena[self].first_child.take();
        arena[self].last_child = None;

        let mut child_opt = first;
        while let Some(child) = child_opt {
            let next = arena[child].next_sibling.take();
            arena[child].previous_sibling = None;
            arena[child].parent = None;
            child_opt = next;
        }
    }

    /// Removes all children of this node from the arena, returning an
    /// error on failure.
    ///
    /// # Failures
    ///
    /// Returns [`NodeError::Removed`] if the node has been removed or the
    /// ID is stale.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let root = arena.new_node("root");
    /// root.append_value("c1", &mut arena);
    /// assert!(root.checked_remove_children(&mut arena).is_ok());
    /// assert_eq!(root.children(&arena).count(), 0);
    /// ```
    pub fn checked_remove_children<T>(self, arena: &mut Arena<T>) -> Result<(), NodeError> {
        if self.is_removed(arena) {
            return Err(NodeError::Removed);
        }
        self.remove_children(arena);
        Ok(())
    }

    /// Removes all children of this node from the arena, keeping the
    /// node itself in its current position.
    ///
    /// This is equivalent to calling [`remove_subtree`] on each child.
    ///
    /// # Panics
    ///
    /// Panics if the node ID is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1");
    /// # let n1_1 = arena.new_node("1_1");
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2");
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_2_1 = arena.new_node("1_2_1");
    /// # n1_2.append(n1_2_1, &mut arena);
    /// # let n1_3 = arena.new_node("1_3");
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_2
    /// //     |   `-- 1_2_1
    /// //     `-- 1_3
    ///
    /// n1.remove_children(&mut arena);
    ///
    /// // arena
    /// // `-- 1
    ///
    /// assert_eq!(n1.children(&arena).count(), 0);
    /// assert!(n1_1.is_removed(&arena));
    /// assert!(n1_2.is_removed(&arena));
    /// assert!(n1_2_1.is_removed(&arena));
    /// assert!(n1_3.is_removed(&arena));
    /// ```
    ///
    /// [`remove_subtree`]: NodeId::remove_subtree
    pub fn remove_children<T>(self, arena: &mut Arena<T>) {
        let first = arena[self].first_child.take();
        arena[self].last_child = None;

        let mut cursor = first;
        while let Some(id) = cursor {
            let node = &arena[id];
            let first_child = node.first_child;
            let next_sibling = node.next_sibling;
            let parent = node.parent;
            arena.free_node(id);
            cursor = first_child.or(next_sibling).or_else(|| {
                let mut ancestor = parent;
                while let Some(a) = ancestor {
                    if a == self {
                        return None;
                    }
                    let ancestor_node = &arena[a];
                    if let Some(sib) = ancestor_node.next_sibling {
                        return Some(sib);
                    }
                    ancestor = ancestor_node.parent;
                }
                None
            });
        }
    }

    /// Moves this node (and its subtree) to become the last child of
    /// `new_parent`, returning an error on failure.
    ///
    /// # Failures
    ///
    /// Returns the same errors as [`checked_append`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let a = arena.new_node("a");
    /// let b = a.append_value("b", &mut arena);
    /// let c = arena.new_node("c");
    /// assert!(b.checked_reparent(c, &mut arena).is_ok());
    /// assert_eq!(b.parent(&arena), Some(c));
    /// ```
    ///
    /// [`checked_append`]: NodeId::checked_append
    pub fn checked_reparent<T>(
        self,
        new_parent: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        new_parent.checked_append(self, arena)
    }

    /// Moves this node (and its subtree) to become the last child of
    /// `new_parent`.
    ///
    /// This is a convenience wrapper around [`detach`] followed by
    /// [`append`].
    ///
    /// # Panics
    ///
    /// Panics if `new_parent` is `self` or a descendant of `self`, or
    /// if either node has been removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let a = arena.new_node("a");
    /// let b = a.append_value("b", &mut arena);
    /// let c = arena.new_node("c");
    ///
    /// b.reparent(c, &mut arena);
    ///
    /// assert_eq!(b.parent(&arena), Some(c));
    /// assert_eq!(a.children(&arena).count(), 0);
    /// assert_eq!(c.first_child(&arena), Some(b));
    /// ```
    ///
    /// [`detach`]: NodeId::detach
    /// [`append`]: NodeId::append
    pub fn reparent<T>(self, new_parent: NodeId, arena: &mut Arena<T>) {
        new_parent.append(self, arena);
    }

    /// Returns `true` if the subtree rooted at this node is structurally
    /// equal to the subtree rooted at `other`, comparing node data with
    /// `PartialEq`.
    ///
    /// Two subtrees are equal if they have the same shape and the same
    /// data at every corresponding position.
    ///
    /// The two nodes may be in the same or different arenas.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut a1 = Arena::new();
    /// let r1 = a1.new_node(1);
    /// r1.append_value(2, &mut a1);
    /// r1.append_value(3, &mut a1);
    ///
    /// let mut a2 = Arena::new();
    /// let r2 = a2.new_node(1);
    /// r2.append_value(2, &mut a2);
    /// r2.append_value(3, &mut a2);
    ///
    /// assert!(r1.subtree_eq(r2, &a1, &a2));
    /// ```
    #[must_use]
    pub fn subtree_eq<T: PartialEq>(
        self,
        other: NodeId,
        arena_self: &Arena<T>,
        arena_other: &Arena<T>,
    ) -> bool {
        use crate::NodeEdge;
        let mut iter_a = self.traverse(arena_self);
        let mut iter_b = other.traverse(arena_other);
        loop {
            match (iter_a.next(), iter_b.next()) {
                (None, None) => return true,
                (Some(NodeEdge::Start(a)), Some(NodeEdge::Start(b))) => {
                    if arena_self[a].get() != arena_other[b].get() {
                        return false;
                    }
                }
                (Some(NodeEdge::End(_)), Some(NodeEdge::End(_))) => {}
                _ => return false,
            }
        }
    }

    /// Returns the pretty-printable proxy object to the node and descendants.
    ///
    /// # (No) guarantees
    ///
    /// This is provided mainly for debugging purpose. Note that the output
    /// format is not guaranteed to be stable, and any format changes won't be
    /// considered as breaking changes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// #
    /// # let mut arena = Arena::new();
    /// # let root = arena.new_node("root");
    /// # let n0 = arena.new_node("0");
    /// # root.append(n0, &mut arena);
    /// # let n0_0 = arena.new_node("0\n0");
    /// # n0.append(n0_0, &mut arena);
    /// # let n0_1 = arena.new_node("0\n1");
    /// # n0.append(n0_1, &mut arena);
    /// # let n1 = arena.new_node("1");
    /// # root.append(n1, &mut arena);
    /// # let n2 = arena.new_node("2");
    /// # root.append(n2, &mut arena);
    /// # let n2_0 = arena.new_node("2\n0");
    /// # n2.append(n2_0, &mut arena);
    /// # let n2_0_0 = arena.new_node("2\n0\n0");
    /// # n2_0.append(n2_0_0, &mut arena);
    ///
    /// //  arena
    /// //  `-- "root"
    /// //      |-- "0"
    /// //      |   |-- "0\n0"
    /// //      |   `-- "0\n1"
    /// //      |-- "1"
    /// //      `-- "2"
    /// //          `-- "2\n0"
    /// //              `-- "2\n0\n0"
    ///
    /// let printable = root.debug_pretty_print(&arena);
    ///
    /// let expected_debug = r#""root"
    /// |-- "0"
    /// |   |-- "0\n0"
    /// |   `-- "0\n1"
    /// |-- "1"
    /// `-- "2"
    ///     `-- "2\n0"
    ///         `-- "2\n0\n0""#;
    /// assert_eq!(format!("{:?}", printable), expected_debug);
    ///
    /// let expected_display = r#"root
    /// |-- 0
    /// |   |-- 0
    /// |   |   0
    /// |   `-- 0
    /// |       1
    /// |-- 1
    /// `-- 2
    ///     `-- 2
    ///         0
    ///         `-- 2
    ///             0
    ///             0"#;
    /// assert_eq!(printable.to_string(), expected_display);
    /// ```
    ///
    /// Alternate styles (`{:#?}` and `{:#}`) are also supported.
    ///
    /// ```
    /// # use indextree::Arena;
    /// #
    /// # let mut arena = Arena::new();
    /// # let root = arena.new_node(Ok(42));
    /// # let child = arena.new_node(Err("err"));
    /// # root.append(child, &mut arena);
    ///
    /// //  arena
    /// //  `-- Ok(42)
    /// //      `-- Err("err")
    ///
    /// let printable = root.debug_pretty_print(&arena);
    ///
    /// let expected_debug = r#"Ok(42)
    /// `-- Err("err")"#;
    /// assert_eq!(format!("{:?}", printable), expected_debug);
    ///
    /// let expected_debug_alternate = r#"Ok(
    ///     42,
    /// )
    /// `-- Err(
    ///         "err",
    ///     )"#;
    /// assert_eq!(format!("{:#?}", printable), expected_debug_alternate);
    /// ```
    #[inline]
    #[must_use]
    pub fn debug_pretty_print<'a, T>(&'a self, arena: &'a Arena<T>) -> DebugPrettyPrint<'a, T> {
        DebugPrettyPrint::new(self, arena)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_subtree_complex() {
        // arena
        // `-- 1
        //     |-- 1_1
        //     |-- 1_2
        //     |   |-- 1_2_1
        //     |   |   `-- 1_2_1_1
        //     |   |       `-- 1_2_1_1_1
        //     |   `-- 1_2_2
        //     `-- 1_3
        let mut arena = Arena::new();
        let n1 = arena.new_node("1");
        let n1_1 = arena.new_node("1_1");
        n1.append(n1_1, &mut arena);
        let n1_2 = arena.new_node("1_2");
        n1.append(n1_2, &mut arena);
        let n1_2_1 = arena.new_node("1_2_1");
        n1_2.append(n1_2_1, &mut arena);
        let n1_2_1_1 = arena.new_node("1_2_1_1");
        n1_2_1.append(n1_2_1_1, &mut arena);
        let n1_2_1_1_1 = arena.new_node("1_2_1_1_1");
        n1_2_1_1.append(n1_2_1_1_1, &mut arena);
        let n1_2_2 = arena.new_node("1_2_2");
        n1_2.append(n1_2_2, &mut arena);
        let n1_3 = arena.new_node("1_3");
        n1.append(n1_3, &mut arena);

        n1_2.remove_subtree(&mut arena);

        assert!(!n1.is_removed(&arena));
        assert!(!n1_1.is_removed(&arena));
        assert!(!n1_3.is_removed(&arena));

        assert!(n1_2.is_removed(&arena));
        assert!(n1_2_1.is_removed(&arena));
        assert!(n1_2_1_1.is_removed(&arena));
        assert!(n1_2_1_1_1.is_removed(&arena));
        assert!(n1_2_2.is_removed(&arena));
    }

    #[test]
    fn test_conversions() {
        let mut arena = Arena::new();
        let n1 = arena.new_node("1");
        assert_eq!(usize::from(n1), 1);
        assert_eq!(NonZeroUsize::from(n1), NonZeroUsize::new(1).unwrap());
        assert_eq!(n1.to_string(), "1");
    }
}
