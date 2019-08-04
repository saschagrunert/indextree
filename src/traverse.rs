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
    };
}

#[derive(Clone)]
/// An iterator of references to the ancestors a given node.
pub struct Ancestors<'a, T: 'a> {
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
/// An iterator of references to the siblings before a given node.
pub struct PrecedingSiblings<'a, T: 'a> {
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
/// An iterator of references to the siblings after a given node.
pub struct FollowingSiblings<'a, T: 'a> {
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
/// An iterator of references to the children of a given node.
pub struct Children<'a, T: 'a> {
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
/// An iterator of references to the children of a given node, in reverse order.
pub struct ReverseChildren<'a, T: 'a> {
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
/// An iterator of references to a given node and its descendants, in tree order.
pub struct Descendants<'a, T: 'a>(Traverse<'a, T>);

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

#[derive(Clone)]
/// An iterator of references to a given node and its descendants, in tree
/// order.
pub struct Traverse<'a, T: 'a> {
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
        match next {
            NodeEdge::Start(node) => match self.arena[node].first_child {
                Some(first_child) => Some(NodeEdge::Start(first_child)),
                None => Some(NodeEdge::End(node)),
            },
            NodeEdge::End(node) => {
                if node == self.root {
                    return None;
                }
                let node = &self.arena[node];
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
}

impl<'a, T> Iterator for Traverse<'a, T> {
    type Item = NodeEdge;

    fn next(&mut self) -> Option<NodeEdge> {
        let next = self.next.take()?;
        self.next = self.next_of_next(next);
        Some(next)
    }
}

#[derive(Clone)]
/// An iterator of references to a given node and its descendants, in reverse
/// tree order.
pub struct ReverseTraverse<'a, T: 'a> {
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
        match next {
            NodeEdge::End(node) => match self.arena[node].last_child {
                Some(last_child) => Some(NodeEdge::End(last_child)),
                None => Some(NodeEdge::Start(node)),
            },
            NodeEdge::Start(node) => {
                if node == self.root {
                    return None;
                }
                let node = &self.arena[node];
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

impl<'a, T> Iterator for ReverseTraverse<'a, T> {
    type Item = NodeEdge;

    fn next(&mut self) -> Option<NodeEdge> {
        let next = self.next.take()?;
        self.next = self.next_of_next(next);
        Some(next)
    }
}
