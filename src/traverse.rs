//! Iterators.

use crate::{Arena, Node, NodeId};

macro_rules! impl_node_iterator {
    ($name:ident, $next:expr) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeId;

            fn next(&mut self) -> Option<NodeId> {
                let node = self.node.take()?;
                self.node = $next(&self.arena[node]);
                Some(node)
            }
        }

        impl<'a, T> core::iter::FusedIterator for $name<'a, T> {}
    };
}

#[derive(Clone)]
/// An iterator of the IDs of the ancestors a given node.
pub struct Ancestors<'a, T> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(Ancestors, |node: &Node<T>| node.parent);

impl<'a, T> Ancestors<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            node: Some(current),
        }
    }
}

#[derive(Clone)]
/// An iterator of the IDs of the siblings before a given node.
pub struct PrecedingSiblings<'a, T> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(PrecedingSiblings, |node: &Node<T>| node.previous_sibling);

impl<'a, T> PrecedingSiblings<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            node: Some(current),
        }
    }
}

#[derive(Clone)]
/// An iterator of the IDs of the siblings after a given node.
pub struct FollowingSiblings<'a, T> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(FollowingSiblings, |node: &Node<T>| node.next_sibling);

impl<'a, T> FollowingSiblings<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            node: Some(current),
        }
    }
}

#[derive(Clone)]
/// An iterator of the IDs of the children of a given node, in insertion order.
pub struct Children<'a, T> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(Children, |node: &Node<T>| node.next_sibling);

impl<'a, T> Children<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            node: arena[current].first_child,
        }
    }
}

#[derive(Clone)]
/// An iterator of the IDs of the children of a given node, in reverse insertion order.
pub struct ReverseChildren<'a, T> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(ReverseChildren, |node: &Node<T>| node.previous_sibling);

impl<'a, T> ReverseChildren<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId) -> Self {
        Self {
            arena,
            node: arena[current].last_child,
        }
    }
}

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
                    // been modified during iteration, but silently stoping
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
