//! Tree node and its associated data.
//!
//! Each [`Node`] stores its data along with links to its parent, children,
//! and siblings. Nodes are not accessed directly; instead, use
//! [`NodeId`](crate::NodeId) to index into an [`Arena`](crate::Arena).

#[cfg(not(feature = "std"))]
use core::{fmt, hash};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{fmt, hash};

use crate::{NodeId, id::NodeStamp};

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub(crate) enum NodeData<T> {
    /// The actual data store
    Data(T),
    /// The next free node position.
    NextFree(Option<usize>),
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// A node within a particular `Arena`.
pub struct Node<T> {
    // Keep these private (with read-only accessors) so that we can keep them
    // consistent. E.g. the parent of a node’s child is that node.
    pub(crate) parent: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) stamp: NodeStamp,
    /// The actual data which will be stored within the tree.
    pub(crate) data: NodeData<T>,
}

impl<T> Node<T> {
    /// Returns a reference to the node data.
    ///
    /// # Panics
    ///
    /// Panics if the node has been removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let id = arena.new_node(42);
    /// assert_eq!(*arena[id].get(), 42);
    /// ```
    #[inline]
    pub fn get(&self) -> &T {
        if let NodeData::Data(ref data) = self.data {
            data
        } else {
            panic!("attempted to access a freed node")
        }
    }

    /// Returns a reference to the node data, or `None` if the node has
    /// been removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let id = arena.new_node(42);
    /// assert_eq!(arena[id].try_get(), Some(&42));
    /// ```
    #[inline]
    pub fn try_get(&self) -> Option<&T> {
        if let NodeData::Data(ref data) = self.data {
            Some(data)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the node data.
    ///
    /// # Panics
    ///
    /// Panics if the node has been removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let id = arena.new_node(42);
    /// *arena[id].get_mut() = 99;
    /// assert_eq!(*arena[id].get(), 99);
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        if let NodeData::Data(ref mut data) = self.data {
            data
        } else {
            panic!("attempted to access a freed node")
        }
    }

    /// Returns a mutable reference to the node data, or `None` if the
    /// node has been removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let id = arena.new_node(42);
    /// if let Some(data) = arena[id].try_get_mut() {
    ///     *data = 99;
    /// }
    /// assert_eq!(*arena[id].get(), 99);
    /// ```
    #[inline]
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        if let NodeData::Data(ref mut data) = self.data {
            Some(data)
        } else {
            None
        }
    }

    /// Consumes the node and returns the contained data, or `None` if
    /// the node has been removed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// arena.new_node(42);
    /// arena.new_node(99);
    ///
    /// let values: Vec<_> = arena
    ///     .into_iter()
    ///     .filter_map(|n| n.into_data())
    ///     .collect();
    /// assert_eq!(values, vec![42, 99]);
    /// ```
    pub fn into_data(self) -> Option<T> {
        match self.data {
            NodeData::Data(data) => Some(data),
            NodeData::NextFree(_) => None,
        }
    }

    /// Creates a new `Node` with the default state and the given data.
    pub(crate) fn new(data: T) -> Self {
        Self {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            stamp: NodeStamp::default(),
            data: NodeData::Data(data),
        }
    }

    /// Convert a removed `Node` to normal with default state and given data.
    pub(crate) fn reuse(&mut self, data: T) {
        debug_assert!(matches!(self.data, NodeData::NextFree(_)));
        debug_assert!(self.stamp.is_removed());
        self.stamp.reuse();
        self.parent = None;
        self.previous_sibling = None;
        self.next_sibling = None;
        self.first_child = None;
        self.last_child = None;
        self.data = NodeData::Data(data);
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
    /// assert_eq!(arena[n1].parent(), None);
    /// assert_eq!(arena[n1_1].parent(), Some(n1));
    /// assert_eq!(arena[n1_2].parent(), Some(n1));
    /// assert_eq!(arena[n1_3].parent(), Some(n1));
    /// ```
    #[inline]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// Returns the ID of the first child of this node, unless it has no child.
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
    /// assert_eq!(arena[n1].first_child(), Some(n1_1));
    /// assert_eq!(arena[n1_1].first_child(), None);
    /// assert_eq!(arena[n1_2].first_child(), None);
    /// assert_eq!(arena[n1_3].first_child(), None);
    /// ```
    #[inline]
    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }

    /// Returns the ID of the last child of this node, unless it has no child.
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
    /// assert_eq!(arena[n1].last_child(), Some(n1_3));
    /// assert_eq!(arena[n1_1].last_child(), None);
    /// assert_eq!(arena[n1_2].last_child(), None);
    /// assert_eq!(arena[n1_3].last_child(), None);
    /// ```
    #[inline]
    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }

    /// Returns the ID of the previous sibling of this node, unless it is a
    /// first child.
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
    /// assert_eq!(arena[n1].previous_sibling(), None);
    /// assert_eq!(arena[n1_1].previous_sibling(), None);
    /// assert_eq!(arena[n1_2].previous_sibling(), Some(n1_1));
    /// assert_eq!(arena[n1_3].previous_sibling(), Some(n1_2));
    /// ```
    ///
    /// Note that newly created nodes are independent toplevel nodes, and they
    /// are not siblings by default.
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n2 = arena.new_node("2");
    /// // arena
    /// // |-- (implicit)
    /// // |   `-- 1
    /// // `-- (implicit)
    /// //     `-- 2
    /// assert_eq!(arena[n1].previous_sibling(), None);
    /// assert_eq!(arena[n2].previous_sibling(), None);
    ///
    /// n1.insert_after(n2, &mut arena);
    /// // arena
    /// // `-- (implicit)
    /// //     |-- 1
    /// //     `-- 2
    /// assert_eq!(arena[n1].previous_sibling(), None);
    /// assert_eq!(arena[n2].previous_sibling(), Some(n1));
    /// ```
    #[inline]
    pub fn previous_sibling(&self) -> Option<NodeId> {
        self.previous_sibling
    }

    /// Returns the ID of the next sibling of this node, unless it is a
    /// last child.
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
    /// assert_eq!(arena[n1].next_sibling(), None);
    /// assert_eq!(arena[n1_1].next_sibling(), Some(n1_2));
    /// assert_eq!(arena[n1_2].next_sibling(), Some(n1_3));
    /// assert_eq!(arena[n1_3].next_sibling(), None);
    /// ```
    ///
    /// Note that newly created nodes are independent toplevel nodes, and they
    /// are not siblings by default.
    ///
    /// ```
    /// # use indextree::Arena;
    /// let mut arena = Arena::new();
    /// let n1 = arena.new_node("1");
    /// let n2 = arena.new_node("2");
    /// // arena
    /// // |-- (implicit)
    /// // |   `-- 1
    /// // `-- (implicit)
    /// //     `-- 2
    /// assert_eq!(arena[n1].next_sibling(), None);
    /// assert_eq!(arena[n2].next_sibling(), None);
    ///
    /// n1.insert_after(n2, &mut arena);
    /// // arena
    /// // `-- (implicit)
    /// //     |-- 1
    /// //     `-- 2
    /// assert_eq!(arena[n1].next_sibling(), Some(n2));
    /// assert_eq!(arena[n2].next_sibling(), None);
    /// ```
    #[inline]
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }

    /// Checks if the node is marked as removed.
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
    /// //     |-- 1_2 *
    /// //     `-- 1_3
    /// assert_eq!(arena[n1_1].next_sibling(), Some(n1_2));
    /// assert_eq!(arena[n1_2].parent(), Some(n1));
    /// assert!(!arena[n1_2].is_removed());
    /// assert_eq!(arena[n1_3].previous_sibling(), Some(n1_2));
    ///
    /// n1_2.remove(&mut arena);
    /// // arena
    /// // `-- 1
    /// //     |-- 1_1
    /// //     `-- 1_3
    /// assert_eq!(arena[n1_1].next_sibling(), Some(n1_3));
    /// assert_eq!(arena[n1_2].parent(), None);
    /// assert!(arena[n1_2].is_removed());
    /// assert_eq!(arena[n1_3].previous_sibling(), Some(n1_1));
    /// ```
    #[inline]
    pub fn is_removed(&self) -> bool {
        self.stamp.is_removed()
    }

    /// Checks if the node is detached.
    pub(crate) fn is_detached(&self) -> bool {
        self.parent.is_none() && self.previous_sibling.is_none() && self.next_sibling.is_none()
    }
}

impl<T: hash::Hash> hash::Hash for Node<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.previous_sibling.hash(state);
        self.next_sibling.hash(state);
        self.first_child.hash(state);
        self.last_child.hash(state);
        self.stamp.hash(state);
        match self.data {
            NodeData::Data(ref data) => {
                0u8.hash(state);
                data.hash(state);
            }
            NodeData::NextFree(ref next) => {
                1u8.hash(state);
                next.hash(state);
            }
        }
    }
}

impl<T> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "parent: {parent}; ")?;
        } else {
            write!(f, "no parent; ")?;
        }
        if let Some(previous_sibling) = self.previous_sibling {
            write!(f, "previous sibling: {previous_sibling}; ")?;
        } else {
            write!(f, "no previous sibling; ")?;
        }
        if let Some(next_sibling) = self.next_sibling {
            write!(f, "next sibling: {next_sibling}; ")?;
        } else {
            write!(f, "no next sibling; ")?;
        }
        if let Some(first_child) = self.first_child {
            write!(f, "first child: {first_child}; ")?;
        } else {
            write!(f, "no first child; ")?;
        }
        if let Some(last_child) = self.last_child {
            write!(f, "last child: {last_child}; ")?;
        } else {
            write!(f, "no last child; ")?;
        }
        Ok(())
    }
}
