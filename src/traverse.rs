//! Iterators.

use crate::{Arena, Node, NodeId};

macro_rules! impl_node_iterator {
    ($name:ident, $next:expr) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeId;

            fn next(&mut self) -> Option<NodeId> {
                match self.node.take() {
                    Some(node) => {
                        self.node = $next(&self.arena[node]);
                        Some(node)
                    }
                    None => None,
                }
            }
        }
    };
}

/// An iterator of references to the ancestors a given node.
pub struct Ancestors<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) node: Option<NodeId>,
}
impl_node_iterator!(Ancestors, |node: &Node<T>| node.parent);

/// An iterator of references to the siblings before a given node.
pub struct PrecedingSiblings<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) node: Option<NodeId>,
}
impl_node_iterator!(PrecedingSiblings, |node: &Node<T>| node.previous_sibling);

/// An iterator of references to the siblings after a given node.
pub struct FollowingSiblings<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) node: Option<NodeId>,
}
impl_node_iterator!(FollowingSiblings, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the children of a given node.
pub struct Children<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) node: Option<NodeId>,
}
impl_node_iterator!(Children, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the children of a given node, in reverse order.
pub struct ReverseChildren<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) node: Option<NodeId>,
}
impl_node_iterator!(ReverseChildren, |node: &Node<T>| node.previous_sibling);

/// An iterator of references to a given node and its descendants, in tree
/// order.
pub struct Descendants<'a, T: 'a>(pub(crate) Traverse<'a, T>);

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Indicator if the node is at a start or endpoint of the tree
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children. Yielded by
    /// `Traverse::next` before the node’s descendants. In HTML or XML, this
    /// corresponds to an opening tag like `<div>`
    Start(T),

    /// Indicates that end of a node that has children. Yielded by
    /// `Traverse::next` after the node’s descendants. In HTML or XML, this
    /// corresponds to a closing tag like `</div>`
    End(T),
}

/// An iterator of references to a given node and its descendants, in tree
/// order.
pub struct Traverse<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) root: NodeId,
    pub(crate) next: Option<NodeEdge<NodeId>>,
}

impl<'a, T> Iterator for Traverse<'a, T> {
    type Item = NodeEdge<NodeId>;

    fn next(&mut self) -> Option<NodeEdge<NodeId>> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::Start(node) => match self.arena[node].first_child
                    {
                        Some(first_child) => Some(NodeEdge::Start(first_child)),
                        None => Some(NodeEdge::End(node)),
                    },
                    NodeEdge::End(node) => {
                        if node == self.root {
                            None
                        } else {
                            match self.arena[node].next_sibling {
                                Some(next_sibling) => {
                                    Some(NodeEdge::Start(next_sibling))
                                }
                                None => {
                                    match self.arena[node].parent {
                                        Some(parent) => {
                                            Some(NodeEdge::End(parent))
                                        }

                                        // `node.parent()` here can only be
                                        // `None` if the tree has been modified
                                        // during iteration, but silently
                                        // stoping iteration seems a more
                                        // sensible behavior than panicking.
                                        None => None,
                                    }
                                }
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None,
        }
    }
}

/// An iterator of references to a given node and its descendants, in reverse
/// tree order.
pub struct ReverseTraverse<'a, T: 'a> {
    pub(crate) arena: &'a Arena<T>,
    pub(crate) root: NodeId,
    pub(crate) next: Option<NodeEdge<NodeId>>,
}

impl<'a, T> Iterator for ReverseTraverse<'a, T> {
    type Item = NodeEdge<NodeId>;

    fn next(&mut self) -> Option<NodeEdge<NodeId>> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::End(node) => match self.arena[node].last_child {
                        Some(last_child) => Some(NodeEdge::End(last_child)),
                        None => Some(NodeEdge::Start(node)),
                    },
                    NodeEdge::Start(node) => {
                        if node == self.root {
                            None
                        } else {
                            match self.arena[node].previous_sibling {
                                Some(previous_sibling) => {
                                    Some(NodeEdge::End(previous_sibling))
                                }
                                None => {
                                    match self.arena[node].parent {
                                        Some(parent) => {
                                            Some(NodeEdge::Start(parent))
                                        }

                                        // `node.parent()` here can only be
                                        // `None` if the tree has been modified
                                        // during iteration, but silently
                                        // stoping iteration seems a more
                                        // sensible behavior than panicking.
                                        None => None,
                                    }
                                }
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None,
        }
    }
}

