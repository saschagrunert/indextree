//! Insertion errors.

use indextree::{Arena, NodeError};

#[test]
fn append_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.checked_append(n1, &mut arena).is_err());
}

#[test]
fn prepend_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.checked_prepend(n1, &mut arena).is_err());
}

#[test]
fn insert_after_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.checked_insert_after(n1, &mut arena).is_err());
}

#[test]
fn insert_before_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.checked_insert_before(n1, &mut arena).is_err());
}

#[test]
fn insert_after_ancestor() {
    // root -> n1 -> n2 -> n3
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let n1 = root.append_value("1", &mut arena);
    let n2 = n1.append_value("2", &mut arena);
    let n3 = n2.append_value("3", &mut arena);
    assert_eq!(
        n2.checked_insert_after(n1, &mut arena),
        Err(NodeError::AppendAncestor)
    );
    assert_eq!(
        n3.checked_insert_after(n1, &mut arena),
        Err(NodeError::AppendAncestor)
    );
    assert_eq!(n1.parent(&arena), Some(root));
    assert!(arena.validate());
}

#[test]
fn insert_before_ancestor() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let n1 = root.append_value("1", &mut arena);
    let n2 = n1.append_value("2", &mut arena);
    let n3 = n2.append_value("3", &mut arena);
    assert_eq!(
        n2.checked_insert_before(n1, &mut arena),
        Err(NodeError::PrependAncestor)
    );
    assert_eq!(
        n3.checked_insert_before(n1, &mut arena),
        Err(NodeError::PrependAncestor)
    );
    assert_eq!(n1.parent(&arena), Some(root));
    assert!(arena.validate());
}

#[test]
#[should_panic(expected = "Preconditions not met")]
fn insert_after_ancestor_panics() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n2 = n1.append_value("2", &mut arena);
    let n3 = n2.append_value("3", &mut arena);
    n3.insert_after(n1, &mut arena);
}

#[test]
fn value_insertion_into_removed_node_panics() {
    type Insert = fn(indextree::NodeId, &mut Arena<&str>);
    let inserts: [Insert; 4] = [
        |id, arena| {
            id.append_value("new", arena);
        },
        |id, arena| {
            id.prepend_value("new", arena);
        },
        |id, arena| {
            id.insert_after_value("new", arena);
        },
        |id, arena| {
            id.insert_before_value("new", arena);
        },
    ];
    for insert in inserts {
        let mut arena = Arena::new();
        let n1 = arena.new_node("1");
        n1.remove(&mut arena);
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| insert(n1, &mut arena)));
        assert!(result.is_err());
        // No node got created and attached to the freed slot.
        assert_eq!(arena.len(), 1);
        assert!(arena.validate());
    }
}

#[test]
fn panics_point_at_caller() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    n1.remove(&mut arena);
    let (tx, rx) = std::sync::mpsc::channel();
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = tx.send(info.location().map(|l| l.file().to_string()));
    }));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        n1.append_value("new", &mut arena);
    }));
    std::panic::set_hook(hook);
    assert!(result.is_err());
    assert_eq!(rx.recv().unwrap().as_deref(), Some(file!()));
}
