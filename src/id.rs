//! Node ID.

#[cfg(not(feature = "std"))]
use core::{fmt, num::NonZeroUsize};

use failure::{bail, Fallible};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{fmt, num::NonZeroUsize};

use crate::{
    relations::insert_with_neighbors, siblings_range::SiblingsRange, Ancestors, Arena, Children,
    Descendants, FollowingSiblings, NodeError, PrecedingSiblings, ReverseChildren, ReverseTraverse,
    Traverse,
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
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index1)
    }
}

impl NodeId {
    /// Returns zero-based index.
    pub(crate) fn index0(self) -> usize {
        // This is totally safe because `self.index1 >= 1` is guaranteed by
        // `NonZeroUsize` type.
        self.index1.get() - 1
    }

    /// Creates a new `NodeId` from the given zero-based index.
    ///
    /// # Panics
    ///
    /// Panics if the value is [`usize::max_value()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// let baz = arena.new_node("baz");
    ///
    /// assert_eq!(NodeId::new(0), foo);
    /// assert_eq!(NodeId::new(1), bar);
    /// assert_eq!(NodeId::new(2), baz);
    /// ```
    ///
    /// [`usize::max_value()`]:
    /// https://doc.rust-lang.org/stable/std/primitive.usize.html#method.min_value
    pub fn new(index0: usize) -> Self {
        let index1 = NonZeroUsize::new(index0.wrapping_add(1))
            .expect("Attempt to create `NodeId` from `usize::max_value()`");
        NodeId { index1 }
    }

    /// Creates a new `NodeId` from the given one-based index.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::num::NonZeroUsize;
    /// # use indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// let _baz = arena.new_node("baz");
    ///
    /// let second_id = NonZeroUsize::new(2)
    ///     .expect("Should success with non-zero integer");
    /// let second = NodeId::from_non_zero_usize(second_id);
    /// assert_eq!(second, bar);
    /// ```
    pub fn from_non_zero_usize(index1: NonZeroUsize) -> Self {
        NodeId { index1 }
    }

    /// Returns an iterator of references to this node and its ancestors.
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
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |       `-- 1_1_1_1
    /// //     _-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_1_1.ancestors(&arena);
    /// assert_eq!(iter.next(), Some(n1_1_1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn ancestors<T>(self, arena: &Arena<T>) -> Ancestors<T> {
        Ancestors::new(arena, self)
    }

    /// Returns an iterator of references to this node and the siblings before
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
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_2.preceding_siblings(&arena);
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn preceding_siblings<T>(self, arena: &Arena<T>) -> PrecedingSiblings<T> {
        PrecedingSiblings::new(arena, self)
    }

    /// Returns an iterator of references to this node and the siblings after
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
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1_2.following_siblings(&arena);
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn following_siblings<T>(self, arena: &Arena<T>) -> FollowingSiblings<T> {
        FollowingSiblings::new(arena, self)
    }

    /// Returns an iterator of references to this node’s children.
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
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.children(&arena);
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn children<T>(self, arena: &Arena<T>) -> Children<T> {
        Children::new(arena, self)
    }

    /// Returns an iterator of references to this node’s children, in reverse
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
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.reverse_children(&arena);
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn reverse_children<T>(self, arena: &Arena<T>) -> ReverseChildren<T> {
        ReverseChildren::new(arena, self)
    }

    /// Returns an iterator of references to this node and its descendants, in
    /// tree order.
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
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |       `-- 1_1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_1_1));
    /// assert_eq!(iter.next(), Some(n1_1_1_1));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`skip`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.skip
    pub fn descendants<T>(self, arena: &Arena<T>) -> Descendants<T> {
        Descendants::new(arena, self)
    }

    /// Returns an iterator of references to this node and its descendants, in
    /// tree order.
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
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.traverse(&arena);
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_2)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_2)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_3)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_3)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn traverse<T>(self, arena: &Arena<T>) -> Traverse<T> {
        Traverse::new(arena, self)
    }

    /// Returns an iterator of references to this node and its descendants, in
    /// reverse tree order.
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
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// let mut iter = n1.reverse_traverse(&arena);
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_3)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_3)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_2)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_2)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::End(n1_1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1_1)));
    /// assert_eq!(iter.next(), Some(NodeEdge::Start(n1)));
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
    /// # // arena
    /// # // `-- 1
    /// # //     |-- 1_1
    /// # //     |   `-- 1_1_1
    /// # //     |-- 1_2
    /// # //     `-- 1_3
    /// #
    /// let traverse = n1.traverse(&arena).collect::<Vec<_>>();
    /// let mut reverse = n1.reverse_traverse(&arena).collect::<Vec<_>>();
    /// reverse.reverse();
    /// assert_eq!(traverse, reverse);
    /// ```
    pub fn reverse_traverse<T>(self, arena: &Arena<T>) -> ReverseTraverse<T> {
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
    /// //     `-- 1_2
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
    /// # Failures
    ///
    /// Returns an error if the given new child is `self`.
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
    pub fn append<T>(self, new_child: NodeId, arena: &mut Arena<T>) -> Fallible<()> {
        if new_child == self {
            bail!(NodeError::AppendSelf);
        }
        new_child.detach(arena);
        insert_with_neighbors(arena, new_child, Some(self), arena[self].last_child, None)
            .expect("Should never fail: `new_child` is not `self`");

        Ok(())
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Failures
    ///
    /// Returns an error if the given new child is `self`.
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
    pub fn prepend<T>(self, new_child: NodeId, arena: &mut Arena<T>) -> Fallible<()> {
        if new_child == self {
            bail!(NodeError::PrependSelf);
        }
        insert_with_neighbors(arena, new_child, Some(self), None, arena[self].first_child)
            .expect("Should never fail: `new_child` is not `self`");

        Ok(())
    }

    /// Inserts a new sibling after this node.
    ///
    /// # Failures
    ///
    /// Returns an error if the given new sibling is `self`.
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
    /// //     `-- 1_2
    ///
    /// let n1_3 = arena.new_node("1_3");
    /// n1_1.insert_after(n1_3, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_3
    /// //     `-- 1_2
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn insert_after<T>(self, new_sibling: NodeId, arena: &mut Arena<T>) -> Fallible<()> {
        if new_sibling == self {
            bail!(NodeError::InsertAfterSelf);
        }
        new_sibling.detach(arena);
        let (next_sibling, parent) = {
            let current = &arena[self];
            (current.next_sibling, current.parent)
        };
        insert_with_neighbors(arena, new_sibling, parent, Some(self), next_sibling)
            .expect("Should never fail: `new_sibling` is not `self`");

        Ok(())
    }

    /// Inserts a new sibling before this node.
    ///
    /// # Failures
    ///
    /// Returns an error if the given new sibling is `self`.
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
    /// //     `-- 1_2
    ///
    /// let n1_3 = arena.new_node("1_3");
    /// n1_2.insert_before(n1_3, &mut arena);
    ///
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |-- 1_3
    /// //     `-- 1_2
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), Some(n1_2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn insert_before<T>(self, new_sibling: NodeId, arena: &mut Arena<T>) -> Fallible<()> {
        if new_sibling == self {
            bail!(NodeError::InsertBeforeSelf);
        }
        new_sibling.detach(arena);
        let (previous_sibling, parent) = {
            let current = &arena[self];
            (current.previous_sibling, current.parent)
        };
        insert_with_neighbors(arena, new_sibling, parent, previous_sibling, Some(self))
            .expect("Should never fail: `new_sibling` is not `self`");

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
    /// //     |-- 1_2
    /// //     |   |-- 1_2_1
    /// //     |   `-- 1_2_2
    /// //     `-- 1_3
    ///
    /// n1_2.remove(&mut arena)?;
    ///
    /// let mut iter = n1.descendants(&arena);
    /// assert_eq!(iter.next(), Some(n1));
    /// assert_eq!(iter.next(), Some(n1_1));
    /// assert_eq!(iter.next(), Some(n1_2_1));
    /// assert_eq!(iter.next(), Some(n1_2_2));
    /// assert_eq!(iter.next(), Some(n1_3));
    /// assert_eq!(iter.next(), None);
    /// # Ok::<(), failure::Error>(())
    /// ```
    ///
    /// [`Node::is_removed()`]: struct.Node.html#method.is_removed
    pub fn remove<T>(self, arena: &mut Arena<T>) -> Fallible<()> {
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
        arena[self].removed = true;
        debug_assert!(arena[self].is_detached());

        Ok(())
    }
}
