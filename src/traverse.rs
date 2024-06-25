//! Iterators.

#![allow(clippy::redundant_closure_call)]

use crate::{Arena, Node, NodeId};

#[derive(Clone)]
struct Iter<'a, T> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}

impl<'a, T> Iter<'a, T> {
    fn new(arena: &'a Arena<T>, node: impl Into<Option<NodeId>>) -> Self {
        let node = node.into();

        Self { arena, node }
    }
}

#[derive(Clone)]
struct DoubleEndedIter<'a, T> {
    arena: &'a Arena<T>,
    head: Option<NodeId>,
    tail: Option<NodeId>,
}

impl<'a, T> DoubleEndedIter<'a, T> {
    fn new(
        arena: &'a Arena<T>,
        head: impl Into<Option<NodeId>>,
        tail: impl Into<Option<NodeId>>,
    ) -> Self {
        let head = head.into();
        let tail = tail.into();

        Self { arena, head, tail }
    }
}

macro_rules! new_iterator {
    ($(#[$attr:meta])* $name:ident, inner = $inner:ident, new = $new:expr $(,)?) => {
        $(#[$attr])*
        #[repr(transparent)]
        #[derive(Clone)]
        pub struct $name<'a, T>($inner<'a, T>);

        #[allow(deprecated)]
        impl<'a, T> $name<'a, T> {
            pub(crate) fn new(arena: &'a Arena<T>, node: NodeId) -> Self {
                let new: fn(&'a Arena<T>, NodeId) -> $inner<'a, T> = $new;
                Self(new(arena, node))
            }
        }
    };
    ($(#[$attr:meta])* $name:ident, new = $new:expr, next = $next:expr $(,)?) => {
        new_iterator!(
            $(#[$attr])*
            $name,
            inner = Iter,
            new = $new,
        );

        #[allow(deprecated)]
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeId;

            fn next(&mut self) -> Option<NodeId> {
                let next: fn(&Node<T>) -> Option<NodeId> = $next;

                let node = self.0.node.take()?;
                self.0.node = next(&self.0.arena[node]);
                Some(node)
            }
        }

        #[allow(deprecated)]
        impl<'a, T> core::iter::FusedIterator for $name<'a, T> {}
    };
    ($(#[$attr:meta])* $name:ident, new = $new:expr, next = $next:expr, next_back = $next_back:expr $(,)?) => {
        new_iterator!(
            $(#[$attr])*
            $name,
            inner = DoubleEndedIter,
            new = $new,
        );

        #[allow(deprecated)]
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeId;

            fn next(&mut self) -> Option<NodeId> {
                match (self.0.head, self.0.tail) {
                    (Some(head), Some(tail)) if head == tail => {
                        let result = head;
                        self.0.head = None;
                        self.0.tail = None;
                        Some(result)
                    }
                    (Some(head), None) | (Some(head), Some(_)) => {
                        let next: fn(&Node<T>) -> Option<NodeId> = $next;

                        self.0.head = next(&self.0.arena[head]);
                        Some(head)
                    }
                    (None, Some(_)) | (None, None) => None,
                }
            }
        }

        #[allow(deprecated)]
        impl<'a, T> ::core::iter::DoubleEndedIterator for $name<'a, T> {
            fn next_back(&mut self) -> Option<Self::Item> {
                match (self.0.head, self.0.tail) {
                    (Some(head), Some(tail)) if head == tail => {
                        let result = head;
                        self.0.head = None;
                        self.0.tail = None;
                        Some(result)
                    }
                    (None, Some(tail)) | (Some(_), Some(tail)) => {
                        let next_back: fn(&Node<T>) -> Option<NodeId> = $next_back;

                        self.0.head = next_back(&self.0.arena[tail]);
                        Some(tail)
                    }
                    (Some(_), None)| (None, None) => None,
                }
            }
        }
    };
    ($(#[$attr:meta])* $name:ident, next = $next:expr $(,)?) => {
        new_iterator!(
            $(#[$attr])*
            $name,
            new = |arena, node| Iter::new(arena, node),
            next = $next,
        );
    };
    ($(#[$attr:meta])* $name:ident, next = $next:expr, next_back = $next_back:expr $(,)?) => {
        new_iterator!(
            $(#[$attr])*
            $name,
            new = |arena, node| DoubleEndedIter::new(arena, node, None),
            next = $next,
            next_back = $next_back,
        );
    };
}

new_iterator!(
    /// An iterator of the IDs of the ancestors of a given node.
    Ancestors,
    next = |node| node.parent,
);

new_iterator!(
    /// An iterator of the IDs of the predecessors of a given node.
    Predecessors,
    next = |node| node.previous_sibling.or(node.parent),
);

new_iterator!(
    /// An iterator of the IDs of the siblings before a given node.
    PrecedingSiblings,
    new = |arena, node| {
        let first = arena
            .get(node)
            .unwrap()
            .parent
            .map(|parent_id| arena.get(parent_id))
            .flatten()
            .map(|parent| parent.first_child)
            .flatten();

        DoubleEndedIter::new(arena, node, first)
    },
    next = |head| head.previous_sibling,
    next_back = |tail| tail.next_sibling,
);

new_iterator!(
    /// An iterator of the IDs of the siblings after a given node.
    FollowingSiblings,
    new = |arena, node| {
        let last = arena
            .get(node)
            .unwrap()
            .parent
            .map(|parent_id| arena.get(parent_id))
            .flatten()
            .map(|parent| parent.last_child)
            .flatten();

        DoubleEndedIter::new(arena, node, last)
    },
    next = |head| head.next_sibling,
    next_back = |tail| tail.previous_sibling,
);

new_iterator!(
    /// An iterator of the IDs of the children of a given node, in insertion order.
    Children,
    new = |arena, node| DoubleEndedIter::new(arena, arena[node].first_child, arena[node].last_child),
    next = |node| node.next_sibling,
    next_back = |tail| tail.previous_sibling,
);

new_iterator!(
    #[deprecated(
        since = "4.7.0",
        note = "please, use Children::rev() instead if you want to iterate in reverse"
    )]
    /// An iterator of the IDs of the children of a given node, in reverse insertion order.
    ReverseChildren,
    new = |arena, node| Iter::new(arena, arena[node].last_child),
    next = |node| node.previous_sibling,
);

#[derive(Clone)]
/// An iterator of the IDs of a given node and its descendants, as a pre-order depth-first search where children are visited in insertion order.
///
/// i.e. node -> first child -> second child
pub struct Descendants<'a, T>(Traverse<'a, T>);

impl<'a, T> Descendants<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self(Traverse::new(arena, current))
    }
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        self.0.find_map(|edge| match edge {
            NodeEdge::Start(node) => Some(node),
            NodeEdge::End(_) => None,
        })
    }
}

impl<'a, T> core::iter::FusedIterator for Descendants<'a, T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Indicator if the node is at a start or endpoint of the tree
pub enum NodeEdge {
    /// Indicates that start of a node that has children.
    ///
    /// Yielded by `Traverse::next()` before the node’s descendants. In HTML or
    /// XML, this corresponds to an opening tag like `<div>`.
    Start(NodeId),

    /// Indicates that end of a node that has children.
    ///
    /// Yielded by `Traverse::next()` after the node’s descendants. In HTML or
    /// XML, this corresponds to a closing tag like `</div>`
    End(NodeId),
}

impl NodeEdge {
    /// Returns the next `NodeEdge` to be returned by forward depth-first traversal.
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
    /// let steps = std::iter::successors(
    ///     Some(NodeEdge::Start(n1)),
    ///     |current| current.next_traverse(&arena)
    /// )
    ///     .collect::<Vec<_>>();
    /// let traversed_by_iter = n1.traverse(&arena).collect::<Vec<_>>();
    /// assert_eq!(
    ///     steps,
    ///     traversed_by_iter,
    ///     "repeated `.next_traverse()`s emit same events as `NodeId::traverse()` iterator"
    /// );
    /// ```
    ///
    /// `NodeEdge` itself does not borrow an arena, so you can modify the nodes
    /// being traversed.
    ///
    /// ```
    /// # use indextree::{Arena, NodeEdge};
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1".to_owned());
    /// # let n1_1 = arena.new_node("1_1".to_owned());
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1".to_owned());
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2".to_owned());
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3".to_owned());
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena: Arena<String>
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// assert_eq!(*arena[n1].get(), "1");
    /// assert_eq!(*arena[n1_1_1].get(), "1_1_1");
    /// assert_eq!(*arena[n1_3].get(), "1_3");
    ///
    /// let mut next = Some(NodeEdge::Start(n1));
    /// let mut count = 0;
    /// while let Some(current) = next {
    ///     next = current.next_traverse(&arena);
    ///     let current = match current {
    ///         NodeEdge::Start(id) => id,
    ///         NodeEdge::End(_) => continue,
    ///     };
    ///
    ///     arena[current].get_mut().push_str(&format!(" (count={})", count));
    ///     count += 1;
    /// }
    ///
    /// assert_eq!(*arena[n1].get(), "1 (count=0)");
    /// assert_eq!(*arena[n1_1_1].get(), "1_1_1 (count=2)");
    /// assert_eq!(*arena[n1_3].get(), "1_3 (count=4)");
    /// ```
    #[must_use]
    pub fn next_traverse<T>(self, arena: &Arena<T>) -> Option<Self> {
        match self {
            NodeEdge::Start(node) => match arena[node].first_child {
                Some(first_child) => Some(NodeEdge::Start(first_child)),
                None => Some(NodeEdge::End(node)),
            },
            NodeEdge::End(node) => {
                let node = &arena[node];
                match node.next_sibling {
                    Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                    // `node.parent()` here can only be `None` if the tree has
                    // been modified during iteration, but silently stoping
                    // iteration seems a more sensible behavior than panicking.
                    None => node.parent.map(NodeEdge::End),
                }
            }
        }
    }

    /// Returns the previous `NodeEdge` to be returned by forward depth-first traversal.
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
    /// let steps = std::iter::successors(
    ///     Some(NodeEdge::End(n1)),
    ///     |current| current.prev_traverse(&arena)
    /// )
    ///     .collect::<Vec<_>>();
    /// let traversed_by_iter = n1.reverse_traverse(&arena).collect::<Vec<_>>();
    /// assert_eq!(
    ///     steps,
    ///     traversed_by_iter,
    ///     "repeated `.prev_traverse()`s emit same events as \
    ///      `NodeId::reverse_traverse()` iterator"
    /// );
    /// ```
    ///
    /// `NodeEdge` itself does not borrow an arena, so you can modify the nodes
    /// being traversed.
    ///
    /// ```
    /// use indextree::{Arena, NodeEdge};
    ///
    /// # let mut arena = Arena::new();
    /// # let n1 = arena.new_node("1".to_owned());
    /// # let n1_1 = arena.new_node("1_1".to_owned());
    /// # n1.append(n1_1, &mut arena);
    /// # let n1_1_1 = arena.new_node("1_1_1".to_owned());
    /// # n1_1.append(n1_1_1, &mut arena);
    /// # let n1_2 = arena.new_node("1_2".to_owned());
    /// # n1.append(n1_2, &mut arena);
    /// # let n1_3 = arena.new_node("1_3".to_owned());
    /// # n1.append(n1_3, &mut arena);
    /// #
    /// // arena: Arena<String>
    /// // `-- 1
    /// //     |-- 1_1
    /// //     |   `-- 1_1_1
    /// //     |-- 1_2
    /// //     `-- 1_3
    ///
    /// assert_eq!(*arena[n1_3].get(), "1_3");
    /// assert_eq!(*arena[n1_1_1].get(), "1_1_1");
    /// assert_eq!(*arena[n1].get(), "1");
    ///
    /// let mut next = Some(NodeEdge::End(n1_3));
    /// let mut count = 0;
    /// while let Some(current) = next {
    ///     next = current.prev_traverse(&arena);
    ///     let current = match current {
    ///         NodeEdge::Start(id) => id,
    ///         NodeEdge::End(_) => continue,
    ///     };
    ///
    ///     arena[current].get_mut().push_str(&format!(" (count={})", count));
    ///     count += 1;
    /// }
    ///
    /// assert_eq!(*arena[n1_3].get(), "1_3 (count=0)");
    /// assert_eq!(*arena[n1_1_1].get(), "1_1_1 (count=2)");
    /// assert_eq!(*arena[n1].get(), "1 (count=4)");
    /// ```
    #[must_use]
    pub fn prev_traverse<T>(self, arena: &Arena<T>) -> Option<Self> {
        match self {
            NodeEdge::End(node) => match arena[node].last_child {
                Some(last_child) => Some(NodeEdge::End(last_child)),
                None => Some(NodeEdge::Start(node)),
            },
            NodeEdge::Start(node) => {
                let node = &arena[node];
                match node.previous_sibling {
                    Some(previous_sibling) => Some(NodeEdge::End(previous_sibling)),
                    // `node.parent()` here can only be `None` if the tree has
                    // been modified during iteration, but silently stopping
                    // iteration seems a more sensible behavior than panicking.
                    None => node.parent.map(NodeEdge::Start),
                }
            }
        }
    }
}

#[derive(Clone)]
/// An iterator of the "sides" of a node visited during a depth-first pre-order traversal,
/// where node sides are visited start to end and children are visited in insertion order.
///
/// i.e. node.start -> first child -> second child -> node.end
pub struct Traverse<'a, T> {
    arena: &'a Arena<T>,
    root: NodeId,
    next: Option<NodeEdge>,
}

impl<'a, T> Traverse<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            root: current,
            next: Some(NodeEdge::Start(current)),
        }
    }

    /// Calculates the next node.
    fn next_of_next(&self, next: NodeEdge) -> Option<NodeEdge> {
        if next == NodeEdge::End(self.root) {
            return None;
        }
        next.next_traverse(self.arena)
    }

    /// Returns a reference to the arena.
    #[inline]
    #[must_use]
    pub(crate) fn arena(&self) -> &Arena<T> {
        self.arena
    }
}

impl<'a, T> Iterator for Traverse<'a, T> {
    type Item = NodeEdge;

    fn next(&mut self) -> Option<NodeEdge> {
        let next = self.next.take()?;
        self.next = self.next_of_next(next);
        Some(next)
    }
}

impl<'a, T> core::iter::FusedIterator for Traverse<'a, T> {}

#[derive(Clone)]
/// An iterator of the "sides" of a node visited during a depth-first pre-order traversal,
/// where nodes are visited end to start and children are visited in reverse insertion order.
///
/// i.e. node.end -> second child -> first child -> node.start
pub struct ReverseTraverse<'a, T> {
    arena: &'a Arena<T>,
    root: NodeId,
    next: Option<NodeEdge>,
}

impl<'a, T> ReverseTraverse<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            root: current,
            next: Some(NodeEdge::End(current)),
        }
    }

    /// Calculates the next node.
    fn next_of_next(&self, next: NodeEdge) -> Option<NodeEdge> {
        if next == NodeEdge::Start(self.root) {
            return None;
        }
        next.prev_traverse(self.arena)
    }
}

impl<'a, T> Iterator for ReverseTraverse<'a, T> {
    type Item = NodeEdge;

    fn next(&mut self) -> Option<NodeEdge> {
        let next = self.next.take()?;
        self.next = self.next_of_next(next);
        Some(next)
    }
}

impl<'a, T> core::iter::FusedIterator for ReverseTraverse<'a, T> {}
