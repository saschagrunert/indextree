//! Errors.

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::{error, fmt};

#[derive(Debug, Clone, Copy)]
/// Possible node failures.
pub enum NodeError {
    /// Attempt to append a node to itself.
    AppendSelf,
    /// Attempt to prepend a node to itself.
    PrependSelf,
    /// Attempt to insert a node before itself.
    InsertBeforeSelf,
    /// Attempt to insert a node after itself.
    InsertAfterSelf,
    // See <https://github.com/rust-lang/rfcs/blob/master/text/2008-non-exhaustive.md#how-we-do-this-today>.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl NodeError {
    fn as_str(self) -> &'static str {
        match self {
            NodeError::AppendSelf => "Can not append a node to itself",
            NodeError::PrependSelf => "Can not prepend a node to itself",
            NodeError::InsertBeforeSelf => "Can not insert a node before itself",
            NodeError::InsertAfterSelf => "Can not insert a node after itself",
            NodeError::__Nonexhaustive => panic!("`__Nonexhaustive` should not be used"),
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
