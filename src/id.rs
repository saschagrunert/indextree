//! Node ID.

#[cfg(not(feature = "std"))]
use core::{fmt, num::NonZeroUsize};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{fmt, num::NonZeroUsize};

use crate::{
    debug_pretty_print::DebugPrettyPrint, relations::insert_with_neighbors,
    siblings_range::SiblingsRange, Ancestors, Arena, Children, Descendants, FollowingSiblings,
    NodeError, PrecedingSiblings, Predecessors, ReverseChildren, ReverseTraverse, Traverse,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// A node identifier within a particular [`Arena`].
///
/// This ID is used to get [`Node`] references from an [`Arena`].
///
/// [`Arena`]: struct.Arena.html
/// [`Node`]: struct.Node.html
pub struct NodeId {
    /// One-based index.
    index1: NonZeroUsize,
    stamp: NodeStamp,
}

/// A stamp for node reuse, use to detect if the node of a `NodeId` point to
/// is still the same node.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash, Default)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
pub(crate) struct NodeStamp(i16);

impl NodeStamp {
    pub fn is_removed(self) -> bool {
        self.0.is_negative()
    }

    pub fn as_removed(&mut self) {
        debug_assert!(!self.is_removed());
        self.0 = if self.0 < i16::MAX {
            -self.0 - 1
        } else {
            -self.0
        };
    }

    pub fn reuseable(self) -> bool {
        debug_assert!(self.is_removed());
        self.0 > i16::MIN
    }

    pub fn reuse(&mut self) -> Self {
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

#[allow(clippy::from_over_into)]
impl Into<NonZeroUsize> for NodeId {
    fn into(self) -> NonZeroUsize {
        self.index1
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for NodeId {
    fn into(self) -> usize {
        self.index1.get()
    }
}

impl NodeId {
    /// Returns zero-based index.
    pub(crate) fn index0(self) -> usize {
        // This is totally safe because `self.index1 >= 1` is guaranteed by
        // `NonZeroUsize` type.
        self.index1.get() - 1
    }

    /// Creates a new `NodeId` from the given one-based index.
    pub(crate) fn from_non_zero_usize(index1: NonZeroUsize, stamp: NodeStamp) -> Self {
        NodeId { index1, stamp }
    }

    /// Return if the `Node` of NodeId point to is removed.
    pub fn is_removed<T>(self, arena: &Arena<T>) -> bool {
        arena[self].stamp != self.stamp
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
    /// //     _-- 1_2
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
    /// //     _-- 1_2
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
    /// //     _-- 1_3
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

    /// Returns an iterator of IDs of this node’s children, in reverse
    /// order.
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
    /// //     |-- 1_1                                          // #3
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2                                          // #2
    /// //     `-- 1_3                                          // #1
    ///
    /// let mut iter = n1.reverse_children(&arena);
    /// assert_eq!(iter.next(), Some(n1_3));                    // #1
    /// assert_eq!(iter.next(), Some(n1_2));                    // #2
    /// assert_eq!(iter.next(), Some(n1_1));                    // #3
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn reverse_children<T>(self, arena: &Arena<T>) -> ReverseChildren<'_, T> {
        ReverseChildren::new(arena, self)
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`remove`]: struct.NodeId.html#method.remove
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`NodeError::AppendSelf`]: enum.NodeError.html#variant.AppendSelf
    /// [`NodeError::Removed`]: enum.NodeError.html#variant.Removed
    /// [`remove`]: struct.NodeId.html#method.remove
    pub fn checked_append<T>(
        self,
        new_child: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::AppendSelf);
        }
        if arena[self].is_removed() || arena[new_child].is_removed() {
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`remove`]: struct.NodeId.html#method.remove
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`NodeError::PrependSelf`]: enum.NodeError.html#variant.PrependSelf
    /// [`NodeError::Removed`]: enum.NodeError.html#variant.Removed
    /// [`remove`]: struct.NodeId.html#method.remove
    pub fn checked_prepend<T>(
        self,
        new_child: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_child == self {
            return Err(NodeError::PrependSelf);
        }
        if arena[self].is_removed() || arena[new_child].is_removed() {
            return Err(NodeError::Removed);
        }
        if self.ancestors(arena).any(|ancestor| new_child == ancestor) {
            return Err(NodeError::PrependAncestor);
        }
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`remove`]: struct.NodeId.html#method.remove
    pub fn insert_after<T>(self, new_sibling: NodeId, arena: &mut Arena<T>) {
        self.checked_insert_after(new_sibling, arena)
            .expect("Preconditions not met: invalid argument");
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`NodeError::InsertAfterSelf`]: enum.NodeError.html#variant.InsertAfterSelf
    /// [`NodeError::Removed`]: enum.NodeError.html#variant.Removed
    /// [`remove`]: struct.NodeId.html#method.remove
    pub fn checked_insert_after<T>(
        self,
        new_sibling: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_sibling == self {
            return Err(NodeError::InsertAfterSelf);
        }
        if arena[self].is_removed() || arena[new_sibling].is_removed() {
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`remove`]: struct.NodeId.html#method.remove
    pub fn insert_before<T>(self, new_sibling: NodeId, arena: &mut Arena<T>) {
        self.checked_insert_before(new_sibling, arena)
            .expect("Preconditions not met: invalid argument");
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
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    /// [`NodeError::InsertBeforeSelf`]: enum.NodeError.html#variant.InsertBeforeSelf
    /// [`NodeError::Removed`]: enum.NodeError.html#variant.Removed
    /// [`remove`]: struct.NodeId.html#method.remove
    pub fn checked_insert_before<T>(
        self,
        new_sibling: NodeId,
        arena: &mut Arena<T>,
    ) -> Result<(), NodeError> {
        if new_sibling == self {
            return Err(NodeError::InsertBeforeSelf);
        }
        if arena[self].is_removed() || arena[new_sibling].is_removed() {
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

    /// Removes a node from the arena.
    ///
    /// Children of the removed node will be inserted to the place where the
    /// removed node was.
    ///
    /// Please note that the node will not be removed from the internal arena
    /// storage, but marked as `removed`. Traversing the arena returns a
    /// plain iterator and contains removed elements too.
    ///
    /// To check if the node is removed or not, use [`Node::is_removed()`].
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
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
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

    /// Removes a node and its descendants from the arena.
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

        // use a preorder traversal to remove node.
        let mut cursor = Some(self);
        while let Some(id) = cursor {
            arena.free_node(id);
            let node = &arena[id];
            cursor = node.first_child.or(node.next_sibling).or_else(|| {
                id.ancestors(arena) // traverse ancestors upwards
                    .skip(1) // skip the starting node itself
                    .find(|n| arena[*n].next_sibling.is_some()) // first ancestor with a sibling
                    .and_then(|n| arena[n].next_sibling) // the sibling is the new cursor
            });
        }
    }

    /// Returns the pretty-printable proxy object to the node and descendants.
    ///
    /// # (No) guarantees
    ///
    /// This is provided mainly for debugging purpose. Node that the output
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
}
