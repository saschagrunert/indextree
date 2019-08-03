//! Node ID.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use core::{fmt, mem, num::NonZeroUsize};

use failure::{bail, Fallible};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{fmt, mem, num::NonZeroUsize};

use crate::{
    Ancestors, Arena, Children, Descendants, FollowingSiblings, GetPairMut, NodeEdge, NodeError,
    PrecedingSiblings, ReverseChildren, ReverseTraverse, Traverse,
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
        Ancestors {
            arena,
            node: Some(self),
        }
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
        PrecedingSiblings {
            arena,
            node: Some(self),
        }
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
        FollowingSiblings {
            arena,
            node: Some(self),
        }
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
        Children {
            arena,
            node: arena[self].first_child,
        }
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
        ReverseChildren {
            arena,
            node: arena[self].last_child,
        }
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
        Descendants(self.traverse(arena))
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
        Traverse {
            arena,
            root: self,
            next: Some(NodeEdge::Start(self)),
        }
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
        ReverseTraverse {
            arena,
            root: self,
            next: Some(NodeEdge::End(self)),
        }
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
        let (parent, previous_sibling, next_sibling) = {
            let node = &mut arena[self];
            (
                node.parent.take(),
                node.previous_sibling.take(),
                node.next_sibling.take(),
            )
        };

        if let Some(next_sibling) = next_sibling {
            arena[next_sibling].previous_sibling = previous_sibling;
        } else if let Some(parent) = parent {
            arena[parent].last_child = previous_sibling;
        }

        if let Some(previous_sibling) = previous_sibling {
            arena[previous_sibling].next_sibling = next_sibling;
        } else if let Some(parent) = parent {
            arena[parent].first_child = next_sibling;
        }
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
        new_child.detach(arena);
        let last_child_opt;
        {
            if let Some((self_borrow, new_child_borrow)) =
                arena.nodes.get_tuple_mut(self.index0(), new_child.index0())
            {
                new_child_borrow.parent = Some(self);
                last_child_opt = mem::replace(&mut self_borrow.last_child, Some(new_child));
                if let Some(last_child) = last_child_opt {
                    new_child_borrow.previous_sibling = Some(last_child);
                } else {
                    assert!(
                        self_borrow.first_child.is_none(),
                        "`first_child` must be `None` if `last_child` was `None`"
                    );
                    self_borrow.first_child = Some(new_child);
                }
            } else {
                bail!(NodeError::AppendSelf);
            }
        }
        if let Some(last_child) = last_child_opt {
            assert!(
                arena[last_child].next_sibling.is_none(),
                "The last child must not have next sibling"
            );
            arena[last_child].next_sibling = Some(new_child);
        }
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
        new_child.detach(arena);
        let first_child_opt;
        {
            if let Some((self_borrow, new_child_borrow)) =
                arena.nodes.get_tuple_mut(self.index0(), new_child.index0())
            {
                new_child_borrow.parent = Some(self);
                first_child_opt = mem::replace(&mut self_borrow.first_child, Some(new_child));
                if let Some(first_child) = first_child_opt {
                    new_child_borrow.next_sibling = Some(first_child);
                } else {
                    assert!(
                        self_borrow.last_child.is_none(),
                        "`last_child` must be `None` if `first_child` was `None`"
                    );
                    self_borrow.last_child = Some(new_child);
                }
            } else {
                bail!(NodeError::PrependSelf);
            }
        }
        if let Some(first_child) = first_child_opt {
            assert!(
                arena[first_child].previous_sibling.is_none(),
                "The last child must not have next sibling"
            );
            arena[first_child].previous_sibling = Some(new_child);
        }
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
        new_sibling.detach(arena);
        let next_sibling_opt;
        let parent_opt;
        {
            if let Some((self_borrow, new_sibling_borrow)) = arena
                .nodes
                .get_tuple_mut(self.index0(), new_sibling.index0())
            {
                parent_opt = self_borrow.parent;
                new_sibling_borrow.parent = parent_opt;
                new_sibling_borrow.previous_sibling = Some(self);
                next_sibling_opt = mem::replace(&mut self_borrow.next_sibling, Some(new_sibling));
                if let Some(next_sibling) = next_sibling_opt {
                    new_sibling_borrow.next_sibling = Some(next_sibling);
                }
            } else {
                bail!(NodeError::InsertAfterSelf);
            }
        }
        if let Some(next_sibling) = next_sibling_opt {
            assert_eq!(
                arena[next_sibling].previous_sibling,
                Some(self),
                "The previous sibling of the next sibling must be the current node"
            );
            arena[next_sibling].previous_sibling = Some(new_sibling);
        } else if let Some(parent) = parent_opt {
            assert_eq!(
                arena[parent].last_child,
                Some(self),
                "The last child of the parent mush be the current node"
            );
            arena[parent].last_child = Some(new_sibling);
        }
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
        new_sibling.detach(arena);
        let previous_sibling_opt;
        let parent_opt;
        {
            if let Some((self_borrow, new_sibling_borrow)) = arena
                .nodes
                .get_tuple_mut(self.index0(), new_sibling.index0())
            {
                parent_opt = self_borrow.parent;
                new_sibling_borrow.parent = parent_opt;
                new_sibling_borrow.next_sibling = Some(self);
                previous_sibling_opt =
                    mem::replace(&mut self_borrow.previous_sibling, Some(new_sibling));
                if let Some(previous_sibling) = previous_sibling_opt {
                    new_sibling_borrow.previous_sibling = Some(previous_sibling);
                }
            } else {
                bail!(NodeError::InsertBeforeSelf);
            }
        }
        if let Some(previous_sibling) = previous_sibling_opt {
            assert_eq!(
                arena[previous_sibling].next_sibling,
                Some(self),
                "The next sibling of the previous sibling must be the current node"
            );
            arena[previous_sibling].next_sibling = Some(new_sibling);
        } else if let Some(parent) = parent_opt {
            // The current node is the first child because it has no previous
            // siblings.
            assert_eq!(
                arena[parent].first_child,
                Some(self),
                "The first child of the parent must be the current node"
            );
            arena[parent].first_child = Some(new_sibling);
        }
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
        // Retrieve needed values and detach this node
        let (parent, previous_sibling, next_sibling, first_child, last_child) = {
            let node = &mut arena[self];
            (
                node.parent.take(),
                node.previous_sibling.take(),
                node.next_sibling.take(),
                node.first_child.take(),
                node.last_child.take(),
            )
        };
        // Note that no `is_detached()` assertion here because neighbor nodes
        // are not yet updated consistently.

        // Modify the parents of the childs
        {
            let mut child_opt = first_child;
            while let Some(child_node) = child_opt.map(|id| &mut arena[id]) {
                child_node.parent = parent;
                child_opt = child_node.next_sibling;
            }
        }

        debug_assert_eq!(first_child.is_some(), last_child.is_some());
        // `prev => ???` and `parent->first_child`
        if let Some(previous_sibling) = previous_sibling {
            arena[previous_sibling].next_sibling = first_child.or(next_sibling);
        } else if let Some(parent) = parent {
            arena[parent].first_child = first_child.or(next_sibling);
        }
        // `??? => first_child`
        if let Some(first_child) = first_child {
            arena[first_child].previous_sibling = previous_sibling;
        }
        // `last_child => ???`
        if let Some(last_child) = last_child {
            arena[last_child].next_sibling = next_sibling;
        }
        // `??? => next` and `parent->last_child`
        if let Some(next_sibling) = next_sibling {
            arena[next_sibling].previous_sibling = last_child.or(previous_sibling);
        } else if let Some(parent) = parent {
            arena[parent].last_child = last_child.or(previous_sibling);
        }

        // Cleanup the current node
        {
            let mut_self = &mut arena[self];
            mut_self.removed = true;
        }

        Ok(())
    }
}
