//! Errors.

use failure::Fail;

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
