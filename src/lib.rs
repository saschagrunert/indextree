//! # Arena based tree data structure
//!
//! This arena tree structure is using just a single `Vec` and numerical
//! identifiers (indices in the vector) instead of reference counted pointers
//! like. This means there is no `RefCell` and mutability is handled in a way
//! much more idiomatic to Rust through unique (&mut) access to the arena. The
//! tree can be sent or shared across threads like a `Vec`. This enables
//! general multiprocessing support like parallel tree traversals.  
//! # Example usage
//! ```
//! use indextree::Arena;
//!
//! // Create a new arena
//! let arena = &mut Arena::new();
//!
//! // Add some new nodes to the arena
//! let a = arena.new_node(1);
//! let b = arena.new_node(2);
//!
//! // Append b to a
//! assert!(a.append(b, arena).is_ok());
//! assert_eq!(b.ancestors(arena).into_iter().count(), 2);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use core::{
    cmp::max,
    fmt, mem,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

use failure::{bail, Fail, Fallible};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[cfg(feature = "std")]
use std::{
    cmp::max,
    fmt, mem,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    slice::Iter,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// A node identifier within a particular `Arena`
pub struct NodeId {
    /// One-based index.
    index1: NonZeroUsize,
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index1)
    }
}

#[derive(Debug, Fail)]
/// Possible node failures
pub enum NodeError {
    #[fail(display = "Can not append a node to itself")]
    AppendSelf,

    #[fail(display = "Can not prepend a node to itself")]
    PrependSelf,

    #[fail(display = "Can not insert a node before itself")]
    InsertBeforeSelf,

    #[fail(display = "Can not insert a node after itself")]
    InsertAfterSelf,

    #[fail(display = "First child is already set")]
    FirstChildAlreadySet,

    #[fail(display = "Previous sibling is already set")]
    PreviousSiblingAlreadySet,

    #[fail(display = "Next sibling is already set")]
    NextSiblingAlreadySet,

    #[fail(display = "Previous sibling not equal current node")]
    PreviousSiblingNotSelf,

    #[fail(display = "Next sibling not equal current node")]
    NextSiblingNotSelf,

    #[fail(display = "First child not equal current node")]
    FirstChildNotSelf,

    #[fail(display = "Last child not equal current node")]
    LastChildNotSelf,

    #[fail(display = "Previous sibling is not set")]
    PreviousSiblingNotSet,

    #[fail(display = "Next sibling is not set")]
    NextSiblingNotSet,

    #[fail(display = "First child is not set")]
    FirstChildNotSet,

    #[fail(display = "Last child is not set")]
    LastChildNotSet,
}

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "derive-eq", derive(Eq))]
/// A node within a particular `Arena`
pub struct Node<T> {
    // Keep these private (with read-only accessors) so that we can keep them
    // consistent. E.g. the parent of a node’s child is that node.
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    removed: bool,

    /// The actual data which will be stored within the tree
    pub data: T,
}

impl<T> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "parent: {}; ", parent)?;
        } else {
            write!(f, "no parent; ")?;
        }
        if let Some(previous_sibling) = self.previous_sibling {
            write!(f, "previous sibling: {}; ", previous_sibling)?;
        } else {
            write!(f, "no previous sibling; ")?;
        }
        if let Some(next_sibling) = self.next_sibling {
            write!(f, "next sibling: {}; ", next_sibling)?;
        } else {
            write!(f, "no next sibling; ")?;
        }
        if let Some(first_child) = self.first_child {
            write!(f, "first child: {}; ", first_child)?;
        } else {
            write!(f, "no first child; ")?;
        }
        if let Some(last_child) = self.last_child {
            write!(f, "last child: {}; ", last_child)?;
        } else {
            write!(f, "no last child; ")?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "derive-eq", derive(Eq))]
/// An `Arena` structure containing certain Nodes
pub struct Arena<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Arena<T> {
    /// Create a new empty `Arena`
    pub fn new() -> Arena<T> {
        Arena { nodes: Vec::new() }
    }

    /// Create a new node from its associated data.
    ///
    /// # Panics
    ///
    /// Panics if the arena already has `usize::max_value()` nodes.
    pub fn new_node(&mut self, data: T) -> NodeId {
        let next_index1 = NonZeroUsize::new(self.nodes.len().wrapping_add(1))
            .expect("Too many nodes in the arena");
        self.nodes.push(Node {
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            removed: false,
            data,
        });
        NodeId::from_non_zero_usize(next_index1)
    }

    // Count nodes in arena.
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    // Returns true if arena has no nodes, false otherwise
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Get a reference to the node with the given id if in the arena, None
    /// otherwise.
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.index0())
    }

    /// Get a mutable reference to the node with the given id if in the arena,
    /// None otherwise.
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.index0())
    }

    /// Iterate over all nodes in the arena in storage-order.
    ///
    /// Note that this iterator also contains removed elements, which can be
    /// tested with the `is_removed()` method on the node.
    pub fn iter(&self) -> Iter<Node<T>> {
        self.nodes.iter()
    }
}

#[cfg(feature = "par_iter")]
impl<T: Sync> Arena<T> {
    /// Return an parallel iterator over the whole arena.
    ///
    /// Note that this iterator also contains removed elements, which can be
    /// tested with the `is_removed()` method on the node.
    pub fn par_iter(&self) -> rayon::slice::Iter<Node<T>> {
        self.nodes.par_iter()
    }
}

trait GetPairMut<T> {
    /// Get mutable references to two distinct nodes
    fn get_tuple_mut(&mut self, a: usize, b: usize)
        -> Option<(&mut T, &mut T)>;
}

impl<T> GetPairMut<T> for Vec<T> {
    fn get_tuple_mut(
        &mut self,
        a: usize,
        b: usize,
    ) -> Option<(&mut T, &mut T)> {
        if a == b {
            return None;
        }
        let (xs, ys) = self.split_at_mut(max(a, b));
        if a < b {
            Some((&mut xs[a], &mut ys[0]))
        } else {
            Some((&mut ys[0], &mut xs[b]))
        }
    }
}

impl<T> Index<NodeId> for Arena<T> {
    type Output = Node<T>;

    fn index(&self, node: NodeId) -> &Node<T> {
        &self.nodes[node.index0()]
    }
}

impl<T> IndexMut<NodeId> for Arena<T> {
    fn index_mut(&mut self, node: NodeId) -> &mut Node<T> {
        &mut self.nodes[node.index0()]
    }
}

impl<T> Node<T> {
    /// Return the ID of the parent node, unless this node is the root of the
    /// tree.
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// Return the ID of the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    /// Return the ID of the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    /// Return the ID of the previous sibling of this node, unless it is a
    /// first child.
    pub fn previous_sibling(&self) -> Option<NodeId> {
        self.previous_sibling
    }

    /// Return the ID of the next sibling of this node, unless it is a
    /// last child.
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    /// Check if the node is marked as removed
    pub fn is_removed(&self) -> bool {
        self.removed
    }
}

impl NodeId {
    /// Returns zero-based index.
    fn index0(self) -> usize {
        // This is totally safe because `self.index1 >= 1` is guaranteed by
        // `NonZeroUsize` type.
        self.index1.get() - 1
    }

    /// Create a `NodeId` used for attempting to get `Node`s references from an
    /// `Arena`.
    ///
    /// Note that a zero-based index should be given.
    ///
    /// # Panics
    ///
    /// Panics if the value is `usize::max_value()`.
    pub fn new(index0: usize) -> Self {
        let index1 = NonZeroUsize::new(index0.wrapping_add(1))
            .expect("Attempt to create `NodeId` from `usize::max_value()`");
        NodeId { index1 }
    }

    /// Creates a new `NodeId` from the given one-based index.
    pub fn from_non_zero_usize(index1: NonZeroUsize) -> Self {
        NodeId { index1 }
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors<T>(self, arena: &Arena<T>) -> Ancestors<T> {
        Ancestors {
            arena,
            node: Some(self),
        }
    }

    /// Return an iterator of references to this node and the siblings before
    /// it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings<T>(
        self,
        arena: &Arena<T>,
    ) -> PrecedingSiblings<T> {
        PrecedingSiblings {
            arena,
            node: Some(self),
        }
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings<T>(
        self,
        arena: &Arena<T>,
    ) -> FollowingSiblings<T> {
        FollowingSiblings {
            arena,
            node: Some(self),
        }
    }

    /// Return an iterator of references to this node’s children.
    pub fn children<T>(self, arena: &Arena<T>) -> Children<T> {
        Children {
            arena,
            node: arena[self].first_child,
        }
    }

    /// Return an iterator of references to this node’s children, in reverse
    /// order.
    pub fn reverse_children<T>(self, arena: &Arena<T>) -> ReverseChildren<T> {
        ReverseChildren {
            arena,
            node: arena[self].last_child,
        }
    }

    /// Return an iterator of references to this node and its descendants, in
    /// tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn descendants<T>(self, arena: &Arena<T>) -> Descendants<T> {
        Descendants(self.traverse(arena))
    }

    /// Return an iterator of references to this node and its descendants, in
    /// tree order.
    pub fn traverse<T>(self, arena: &Arena<T>) -> Traverse<T> {
        Traverse {
            arena,
            root: self,
            next: Some(NodeEdge::Start(self)),
        }
    }

    /// Return an iterator of references to this node and its descendants, in
    /// tree order.
    pub fn reverse_traverse<T>(self, arena: &Arena<T>) -> ReverseTraverse<T> {
        ReverseTraverse {
            arena,
            root: self,
            next: Some(NodeEdge::End(self)),
        }
    }

    /// Detach a node from its parent and siblings. Children are not affected.
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

    /// Append a new child to this node, after existing children.
    pub fn append<T>(
        self,
        new_child: NodeId,
        arena: &mut Arena<T>,
    ) -> Fallible<()> {
        new_child.detach(arena);
        let last_child_opt;
        {
            if let Some((self_borrow, new_child_borrow)) =
                arena.nodes.get_tuple_mut(self.index0(), new_child.index0())
            {
                new_child_borrow.parent = Some(self);
                last_child_opt =
                    mem::replace(&mut self_borrow.last_child, Some(new_child));
                if let Some(last_child) = last_child_opt {
                    new_child_borrow.previous_sibling = Some(last_child);
                } else {
                    if self_borrow.first_child.is_some() {
                        bail!(NodeError::FirstChildAlreadySet);
                    }
                    self_borrow.first_child = Some(new_child);
                }
            } else {
                bail!(NodeError::AppendSelf);
            }
        }
        if let Some(last_child) = last_child_opt {
            if arena[last_child].next_sibling.is_some() {
                bail!(NodeError::NextSiblingAlreadySet);
            }
            arena[last_child].next_sibling = Some(new_child);
        }
        Ok(())
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend<T>(
        self,
        new_child: NodeId,
        arena: &mut Arena<T>,
    ) -> Fallible<()> {
        new_child.detach(arena);
        let first_child_opt;
        {
            if let Some((self_borrow, new_child_borrow)) =
                arena.nodes.get_tuple_mut(self.index0(), new_child.index0())
            {
                new_child_borrow.parent = Some(self);
                first_child_opt =
                    mem::replace(&mut self_borrow.first_child, Some(new_child));
                if let Some(first_child) = first_child_opt {
                    new_child_borrow.next_sibling = Some(first_child);
                } else {
                    self_borrow.last_child = Some(new_child);
                    if self_borrow.first_child.is_some() {
                        bail!(NodeError::FirstChildAlreadySet);
                    }
                }
            } else {
                bail!(NodeError::PrependSelf);
            }
        }
        if let Some(first_child) = first_child_opt {
            if arena[first_child].previous_sibling.is_some() {
                bail!(NodeError::PreviousSiblingAlreadySet);
            }
            arena[first_child].previous_sibling = Some(new_child);
        }
        Ok(())
    }

    /// Insert a new sibling after this node.
    pub fn insert_after<T>(
        self,
        new_sibling: NodeId,
        arena: &mut Arena<T>,
    ) -> Fallible<()> {
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
                next_sibling_opt = mem::replace(
                    &mut self_borrow.next_sibling,
                    Some(new_sibling),
                );
                if let Some(next_sibling) = next_sibling_opt {
                    new_sibling_borrow.next_sibling = Some(next_sibling);
                }
            } else {
                bail!(NodeError::InsertAfterSelf);
            }
        }
        if let Some(next_sibling) = next_sibling_opt {
            if let Some(previous_sibling) = arena[next_sibling].previous_sibling
            {
                if previous_sibling != self {
                    bail!(NodeError::PreviousSiblingNotSelf);
                }
            } else {
                bail!(NodeError::PreviousSiblingNotSet);
            }
            arena[next_sibling].previous_sibling = Some(new_sibling);
        } else if let Some(parent) = parent_opt {
            if let Some(last_child) = arena[parent].last_child {
                if last_child != self {
                    bail!(NodeError::LastChildNotSelf);
                }
            } else {
                bail!(NodeError::LastChildNotSet);
            }
            arena[parent].last_child = Some(new_sibling);
        }
        Ok(())
    }

    /// Insert a new sibling before this node.
    /// success.
    pub fn insert_before<T>(
        self,
        new_sibling: NodeId,
        arena: &mut Arena<T>,
    ) -> Fallible<()> {
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
                previous_sibling_opt = mem::replace(
                    &mut self_borrow.previous_sibling,
                    Some(new_sibling),
                );
                if let Some(previous_sibling) = previous_sibling_opt {
                    new_sibling_borrow.previous_sibling =
                        Some(previous_sibling);
                }
            } else {
                bail!(NodeError::InsertBeforeSelf);
            }
        }
        if let Some(previous_sibling) = previous_sibling_opt {
            if let Some(next_sibling) = arena[previous_sibling].next_sibling {
                if next_sibling != self {
                    bail!(NodeError::NextSiblingNotSelf);
                }
            } else {
                bail!(NodeError::NextSiblingNotSet);
            }
            arena[previous_sibling].next_sibling = Some(new_sibling);
        } else if let Some(parent) = parent_opt {
            if let Some(first_child) = arena[parent].first_child {
                if first_child != self {
                    bail!(NodeError::FirstChildNotSelf);
                }
            } else {
                bail!(NodeError::FirstChildNotSet);
            }
            arena[parent].first_child = Some(new_sibling);
        }
        Ok(())
    }

    /// Remove a node from the arena. Available children of the removed node
    /// will be append to the parent after the previous sibling if available.
    ///
    /// Please note that the node will not be removed from the internal arena
    /// storage, but marked as `removed`. Traversing the arena returns a
    /// plain iterator and contains removed elements too.
    pub fn remove<T>(self, arena: &mut Arena<T>) -> Fallible<()> {
        // Modify the parents of the childs
        for child in self.children(arena).collect::<Vec<_>>() {
            arena[child].parent = arena[self].parent
        }

        // Retrieve needed values
        let (previous_sibling, next_sibling, first_child, last_child) = {
            let node = &mut arena[self];
            (
                node.previous_sibling.take(),
                node.next_sibling.take(),
                node.first_child.take(),
                node.last_child.take(),
            )
        };

        // Modify the front
        if let (Some(previous_sibling), Some(first_child)) =
            (previous_sibling, first_child)
        {
            arena[previous_sibling].next_sibling = Some(first_child);
            arena[first_child].previous_sibling = Some(previous_sibling);
        }

        // Modify the back
        if let (Some(next_sibling), Some(last_child)) =
            (next_sibling, last_child)
        {
            arena[next_sibling].previous_sibling = Some(last_child);
            arena[last_child].next_sibling = Some(next_sibling);
        }

        // Cleanup the current node
        self.detach(arena);
        {
            let mut_self = &mut arena[self];
            mut_self.first_child = None;
            mut_self.last_child = None;
            mut_self.removed = true;
        }
        Ok(())
    }
}

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
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(Ancestors, |node: &Node<T>| node.parent);

/// An iterator of references to the siblings before a given node.
pub struct PrecedingSiblings<'a, T: 'a> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(PrecedingSiblings, |node: &Node<T>| node.previous_sibling);

/// An iterator of references to the siblings after a given node.
pub struct FollowingSiblings<'a, T: 'a> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(FollowingSiblings, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the children of a given node.
pub struct Children<'a, T: 'a> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(Children, |node: &Node<T>| node.next_sibling);

/// An iterator of references to the children of a given node, in reverse order.
pub struct ReverseChildren<'a, T: 'a> {
    arena: &'a Arena<T>,
    node: Option<NodeId>,
}
impl_node_iterator!(ReverseChildren, |node: &Node<T>| node.previous_sibling);

/// An iterator of references to a given node and its descendants, in tree
/// order.
pub struct Descendants<'a, T: 'a>(Traverse<'a, T>);

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

#[derive(Debug, Clone)]
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
    arena: &'a Arena<T>,
    root: NodeId,
    next: Option<NodeEdge<NodeId>>,
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
    arena: &'a Arena<T>,
    root: NodeId,
    next: Option<NodeEdge<NodeId>>,
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
