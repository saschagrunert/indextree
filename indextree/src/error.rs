//! Error types for tree operations.
//!
//! [`NodeError`] is returned by the checked variants of tree mutation methods
//! (e.g. [`NodeId::checked_append`](crate::NodeId::checked_append)).

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::{error, fmt};

/// Errors returned by checked tree mutation methods.
///
/// The checked variants ([`NodeId::checked_append`](crate::NodeId::checked_append),
/// [`NodeId::checked_prepend`](crate::NodeId::checked_prepend),
/// [`NodeId::checked_insert_after`](crate::NodeId::checked_insert_after),
/// [`NodeId::checked_insert_before`](crate::NodeId::checked_insert_before))
/// return this error instead of panicking.
///
/// # Examples
///
/// ```
/// use indextree::{Arena, NodeError};
///
/// let mut arena = Arena::new();
/// let n1 = arena.new_node("1");
/// let n2 = arena.new_node("2");
/// n1.append(n2, &mut arena);
///
/// // Appending an ancestor to its descendant fails.
/// assert!(matches!(
///     n2.checked_append(n1, &mut arena),
///     Err(NodeError::AppendAncestor)
/// ));
/// ```
#[derive(Debug, Clone, Copy)]
pub enum NodeError {
    /// Attempted to append a node to itself.
    AppendSelf,
    /// Attempted to prepend a node to itself.
    PrependSelf,
    /// Attempted to insert a node before itself.
    InsertBeforeSelf,
    /// Attempted to insert a node after itself.
    InsertAfterSelf,
    /// Attempted to operate on a removed node, or insert into a removed node.
    Removed,
    /// Attempted to append an ancestor as a child of its descendant,
    /// which would create a cycle.
    AppendAncestor,
    /// Attempted to prepend an ancestor as a child of its descendant,
    /// which would create a cycle.
    PrependAncestor,
}

impl NodeError {
    fn as_str(self) -> &'static str {
        match self {
            NodeError::AppendSelf => "Can not append a node to itself",
            NodeError::PrependSelf => "Can not prepend a node to itself",
            NodeError::InsertBeforeSelf => "Can not insert a node before itself",
            NodeError::InsertAfterSelf => "Can not insert a node after itself",
            NodeError::Removed => "Removed node cannot have any parent, siblings, and children",
            NodeError::AppendAncestor => "Can not append a node to its descendant",
            NodeError::PrependAncestor => "Can not prepend a node to its descendant",
        }
    }
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "std")]
impl error::Error for NodeError {}

/// An error type that represents the given structure or argument is
/// inconsistent or invalid.
// Intended for internal use.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ConsistencyError {
    /// Specified a node as its parent.
    ParentChildLoop,
    /// Specified a node as its sibling.
    SiblingsLoop,
}

impl fmt::Display for ConsistencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsistencyError::ParentChildLoop => f.write_str("Specified a node as its parent"),
            ConsistencyError::SiblingsLoop => f.write_str("Specified a node as its sibling"),
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for ConsistencyError {}
