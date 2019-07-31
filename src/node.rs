//! Node.

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::fmt;

use crate::NodeId;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "derive-eq", derive(Eq))]
/// A node within a particular `Arena`
pub struct Node<T> {
    // Keep these private (with read-only accessors) so that we can keep them
    // consistent. E.g. the parent of a node’s child is that node.
    pub(crate) parent: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) removed: bool,

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
