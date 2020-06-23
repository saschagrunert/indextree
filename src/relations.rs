//! Utilities related to nodes relations.

use crate::{error::ConsistencyError, siblings_range::SiblingsRange, Arena, NodeId};

/// Ensures the given parent, previous, and next nodes are consistent.
///
/// This assert is only enabled in debug build.
macro_rules! debug_assert_triangle_nodes {
    ($arena:expr, $parent:expr, $previous:expr, $next:expr $(,)?) => {{
        if cfg!(debug_assertions) {
            crate::relations::assert_triangle_nodes($arena, $parent, $previous, $next);
        }
    }};
}

/// Ensures the given parent, previous, and next nodes are consistent.
///
/// # Panics
///
/// Panics if the given nodes are inconsistent.
pub(crate) fn assert_triangle_nodes<T>(
    arena: &Arena<T>,
    parent: Option<NodeId>,
    previous: Option<NodeId>,
    next: Option<NodeId>,
) {
    if let Some(previous_node) = previous.map(|id| &arena[id]) {
        assert_eq!(
            previous_node.parent, parent,
            "`prev->parent` must equal to `parent`"
        );
        assert_eq!(
            previous_node.next_sibling, next,
            "`prev->next` must equal to `next`"
        );
    }
    if let Some(next_node) = next.map(|id| &arena[id]) {
        assert_eq!(
            next_node.parent, parent,
            "`next->parent` must equal to `parent`"
        );
        assert_eq!(
            next_node.previous_sibling(&arena),
            previous,
            "`next->prev` must equal to `prev`"
        );
    }
}

/// Connects the given adjacent neighbor nodes and update fields properly.
///
/// This connects the given three nodes (if `Some(_)`) and update fields to make
/// them consistent.
///
/// ```text
///    parent
///     /  \
///    /    \
/// prev -> next
/// ```
///
/// Note that `first_child` and `last_child` fields of the parent may not be
/// updated properly.
/// It is user's responsibility to update them consistent.
pub(crate) fn connect_neighbors<T>(
    arena: &mut Arena<T>,
    parent: Option<NodeId>,
    previous: Option<NodeId>,
    next: Option<NodeId>,
) {
    if cfg!(debug_assertions) {
        if let Some(parent_node) = parent.map(|id| &arena[id]) {
            debug_assert_eq!(
                parent_node.first_child.is_some(),
                parent_node.last_child(&arena).is_some()
            );
            debug_assert!(!parent_node.is_removed());
        }
        debug_assert!(!previous.map_or(false, |id| arena[id].is_removed()));
        debug_assert!(!next.map_or(false, |id| arena[id].is_removed()));
    }

    let (mut parent_first_child, mut parent_last_child) = match parent.map(|id| &arena[id]) {
        Some(node) => (node.first_child, node.last_child(arena)),
        None => match previous.or(next) {
            // NOTE: These are not O(1) operations.
            // This is because nodes are allowed to having no children.
            // If the node has no children, it is impossible to get the first
            // and the last siblings.
            Some(id) => (
                id.preceding_siblings(arena).last(),
                id.following_siblings(arena).last(),
            ),
            None => (None, None),
        },
    };
    if let Some(previous) = previous {
        // `previous` ==> `next`
        arena[previous].next_sibling = next;
        parent_first_child = parent_first_child.or_else(|| Some(previous));
        if parent_first_child == next {
            parent_first_child = Some(previous);
        }
    } else {
        // `next` is the first child of the parent.
        parent_first_child = next;
    }
    if let Some(next) = next {
        // `previous` <== `next`
        // If `previous` is `None`, it means `next` is the first node.
        // Then, `next->cyclic_previous_sibling` will be set later (by
        // `set_children()`), and no need of setting some valid value here.
        if let Some(previous) = previous {
            arena[next].cyclic_previous_sibling = previous;
        }
        parent_last_child = parent_last_child.or_else(|| Some(next));
        if parent_last_child == previous {
            parent_last_child = Some(next);
        }
    } else {
        // `previous` is the last child of the parent.
        parent_last_child = previous;
    }

    let children = match (parent_first_child, parent_last_child) {
        (Some(first), Some(last)) => Some((first, last)),
        (None, None) => None,
        _ => unreachable!(
            "Should never happen because `Some`-ness of \
             `parent_first_child` and `parent_last_child` must be the same"
        ),
    };
    set_children(arena, parent, children);

    debug_assert_triangle_nodes!(arena, parent, previous, next);
}

/// Detaches, inserts, and updates the given node using the given neighbors.
///
/// ```text
/// Before:
///
///    parent
///     /  \
///    /    \
/// prev -> next
///
/// After:
///
///        parent
///    ______/|\_____
///   /       |      \
/// prev -> (new) -> next
/// ```
pub(crate) fn insert_with_neighbors<T>(
    arena: &mut Arena<T>,
    new: NodeId,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
) -> Result<(), ConsistencyError> {
    debug_assert_triangle_nodes!(arena, parent, previous_sibling, next_sibling);
    if previous_sibling == Some(new) || next_sibling == Some(new) {
        // One of the given neighbors is going to be detached.
        return Err(ConsistencyError::SiblingsLoop);
    }
    if parent == Some(new) {
        // The given parent is the node itself.
        return Err(ConsistencyError::ParentChildLoop);
    }

    SiblingsRange::new(new, new)
        .detach_from_siblings(arena)
        .transplant(arena, parent, previous_sibling, next_sibling)
        .expect("Should never fail: neighbors including parent are not `self`");

    debug_assert_triangle_nodes!(arena, parent, previous_sibling, Some(new));
    debug_assert_triangle_nodes!(arena, parent, Some(new), next_sibling);

    Ok(())
}

/// Update the first and the last children and the parent consistently.
fn set_children<T>(
    arena: &mut Arena<T>,
    parent: Option<NodeId>,
    children: Option<(NodeId, NodeId)>,
) {
    let first = match children {
        Some((first, last)) => {
            // Do not check whether `last->cyclic_previous_sibling` is valid
            // here, because they would be temporarily inconsistent in
            // `connect_neighbors()`.
            debug_assert_eq!(arena[last].next_sibling, None);
            arena[first].cyclic_previous_sibling = last;
            Some(first)
        }
        None => None,
    };
    if let Some(parent) = parent {
        arena[parent].first_child = first;
    }
}
