//! Errors.

#[cfg(not(feature = "std"))]
use core::fmt;

use failure::Fail;

#[cfg(feature = "std")]
use std::{error, fmt};

#[derive(Debug, Fail)]
/// Possible node failures.
pub enum NodeError {
    #[fail(display = "Can not append a node to itself")]
    AppendSelf,

    #[fail(display = "Can not prepend a node to itself")]
    PrependSelf,

    #[fail(display = "Can not insert a node before itself")]
    InsertBeforeSelf,

    #[fail(display = "Can not insert a node after itself")]
    InsertAfterSelf,

    // Deprecated, no longer used.
    #[fail(display = "First child is already set")]
    FirstChildAlreadySet,

    // Deprecated, no longer used.
    #[fail(display = "Previous sibling is already set")]
    PreviousSiblingAlreadySet,

    // Deprecated, no longer used.
    #[fail(display = "Next sibling is already set")]
    NextSiblingAlreadySet,

    // Deprecated, no longer used.
    #[fail(display = "Previous sibling not equal current node")]
    PreviousSiblingNotSelf,

    // Deprecated, no longer used.
    #[fail(display = "Next sibling not equal current node")]
    NextSiblingNotSelf,

    // Deprecated, no longer used.
    #[fail(display = "First child not equal current node")]
    FirstChildNotSelf,

    // Deprecated, no longer used.
    #[fail(display = "Last child not equal current node")]
    LastChildNotSelf,

    // Deprecated, no longer used.
    #[fail(display = "Previous sibling is not set")]
    PreviousSiblingNotSet,

    // Deprecated, no longer used.
    #[fail(display = "Next sibling is not set")]
    NextSiblingNotSet,

    // Deprecated, no longer used.
    #[fail(display = "First child is not set")]
    FirstChildNotSet,

    // Deprecated, no longer used.
    #[fail(display = "Last child is not set")]
    LastChildNotSet,
}

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
