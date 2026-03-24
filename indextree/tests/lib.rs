use indextree::{Arena, NodeError};
#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[test]
fn success_create() {
    let mut new_counter = 0;
    let arena = &mut Arena::new();
    macro_rules! new {
        () => {{
            new_counter += 1;
            arena.new_node(new_counter)
        }};
    }

    let a = new!(); // 1
    assert!(a.checked_append(new!(), arena).is_ok()); // 2
    assert!(a.checked_append(new!(), arena).is_ok()); // 3
    assert!(a.checked_prepend(new!(), arena).is_ok()); // 4
    let b = new!(); // 5
    assert!(b.checked_append(a, arena).is_ok());
    assert!(a.checked_insert_before(new!(), arena).is_ok()); // 6
    assert!(a.checked_insert_before(new!(), arena).is_ok()); // 7
    assert!(a.checked_insert_after(new!(), arena).is_ok()); // 8
    assert!(a.checked_insert_after(new!(), arena).is_ok()); // 9
    let c = new!(); // 10
    assert!(b.checked_append(c, arena).is_ok());

    arena[c].previous_sibling().unwrap().detach(arena);

    assert_eq!(
        b.descendants(arena)
            .map(|node| *arena[node].get())
            .collect::<Vec<_>>(),
        [5, 6, 7, 1, 4, 2, 3, 9, 10]
    );
}

#[test]
// Issue #30.
fn first_prepend() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    assert!(a.checked_prepend(b, arena).is_ok());
}

#[test]
fn success_detach() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(1);
    assert!(a.checked_append(b, arena).is_ok());
    assert_eq!(b.ancestors(arena).count(), 2);
    b.detach(arena);
    assert_eq!(b.ancestors(arena).count(), 1);
}

#[test]
fn get() {
    let arena = &mut Arena::new();
    let id = arena.new_node(1);
    assert_eq!(*arena.get(id).unwrap().get(), 1);
}

#[test]
fn get_mut() {
    let arena = &mut Arena::new();
    let id = arena.new_node(1);
    assert_eq!(*arena.get_mut(id).unwrap().get(), 1);
}

#[test]
fn iter() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    assert!(a.checked_append(b, arena).is_ok());
    assert!(b.checked_append(c, arena).is_ok());
    assert!(a.checked_append(d, arena).is_ok());

    let node_refs = arena.iter().collect::<Vec<_>>();
    assert_eq!(node_refs, vec![&arena[a], &arena[b], &arena[c], &arena[d]]);
}

#[test]
fn iter_mut() {
    let arena: &mut Arena<i64> = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    assert!(a.checked_append(b, arena).is_ok());
    assert!(b.checked_append(c, arena).is_ok());
    assert!(a.checked_append(d, arena).is_ok());

    for node in arena.iter_mut() {
        let data = node.get_mut();
        *data = data.wrapping_add(4);
    }

    let node_refs = arena.iter().map(|i| *i.get()).collect::<Vec<_>>();
    assert_eq!(node_refs, vec![5, 6, 7, 8]);
}

#[cfg(feature = "par_iter")]
#[test]
fn par_iter() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    assert!(a.checked_append(b, arena).is_ok());
    assert!(b.checked_append(c, arena).is_ok());
    assert!(a.checked_append(d, arena).is_ok());

    let node_refs = arena.par_iter().collect::<Vec<_>>();
    assert_eq!(node_refs, vec![&arena[a], &arena[b], &arena[c], &arena[d]]);
}

#[test]
fn remove() {
    let arena = &mut Arena::new();
    let n0 = arena.new_node(0);
    let n1 = arena.new_node(1);
    let n2 = arena.new_node(2);
    let n3 = arena.new_node(3);
    let n4 = arena.new_node(4);
    let n5 = arena.new_node(5);
    let n6 = arena.new_node(6);
    assert!(n0.checked_append(n1, arena).is_ok());
    assert!(n0.checked_append(n2, arena).is_ok());
    assert!(n0.checked_append(n3, arena).is_ok());
    assert!(n2.checked_append(n4, arena).is_ok());
    assert!(n2.checked_append(n5, arena).is_ok());
    assert!(n2.checked_append(n5, arena).is_ok());
    assert!(n2.checked_append(n6, arena).is_ok());
    n2.remove(arena);

    let node_refs = arena
        .iter()
        .filter_map(|x| {
            if !x.is_removed() {
                Some(*x.get())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(node_refs, vec![0, 1, 3, 4, 5, 6]);
    assert_eq!(n2.children(arena).count(), 0);
    assert_eq!(n2.descendants(arena).count(), 1);
    assert_eq!(n2.preceding_siblings(arena).count(), 1);
    assert_eq!(n2.following_siblings(arena).count(), 1);

    n3.remove(arena);

    let node_refs = arena
        .iter()
        .filter_map(|x| {
            if !x.is_removed() {
                Some(*x.get())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(node_refs, vec![0, 1, 4, 5, 6]);
    assert_eq!(n3.children(arena).count(), 0);
    assert_eq!(n3.descendants(arena).count(), 1);
    assert_eq!(n3.preceding_siblings(arena).count(), 1);
    assert_eq!(n3.following_siblings(arena).count(), 1);
}

#[test]
fn is_removed() {
    let arena = &mut Arena::new();
    let n0 = arena.new_node(0);
    n0.remove(arena);
    assert!(n0.is_removed(arena));
}

#[test]
fn insert_removed_node() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n2 = arena.new_node("2");
    n2.remove(&mut arena);

    assert!(n1.checked_append(n2, &mut arena).is_err());
    assert!(n2.checked_append(n1, &mut arena).is_err());
    assert!(n1.checked_prepend(n2, &mut arena).is_err());
    assert!(n2.checked_prepend(n1, &mut arena).is_err());
    assert!(n1.checked_insert_after(n2, &mut arena).is_err());
    assert!(n2.checked_insert_after(n1, &mut arena).is_err());
    assert!(n1.checked_insert_before(n2, &mut arena).is_err());
    assert!(n2.checked_insert_before(n1, &mut arena).is_err());
}

#[test]
fn retrieve_node_id() {
    let mut arena = Arena::new();
    let n1_id = arena.new_node("1");
    let n2_id = arena.new_node("2");
    let n3_id = arena.new_node("3");
    let n1 = arena.get(n1_id).unwrap();
    let n2 = arena.get(n2_id).unwrap();
    let n3 = arena.get(n3_id).unwrap();
    let retrieved_n1_id = arena.get_node_id(n1).unwrap();
    let retrieved_n2_id = arena.get_node_id(n2).unwrap();
    let retrieved_n3_id = arena.get_node_id(n3).unwrap();
    assert_eq!(retrieved_n1_id, n1_id);
    assert_eq!(retrieved_n2_id, n2_id);
    assert_eq!(retrieved_n3_id, n3_id);
}

#[test]
// Issue #78.
fn append_ancestor() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let child = arena.new_node("child");
    root.append(child, &mut arena);
    let grandchild = arena.new_node("grandchild");
    child.append(grandchild, &mut arena);
    // root
    // `-- child
    //     `-- grandchild
    assert!(matches!(
        grandchild.checked_append(root, &mut arena),
        Err(NodeError::AppendAncestor)
    ));
    assert!(matches!(
        grandchild.checked_append(child, &mut arena),
        Err(NodeError::AppendAncestor)
    ));
}

#[test]
// Issue #78.
fn prepend_ancestor() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let child = arena.new_node("child");
    root.append(child, &mut arena);
    let grandchild = arena.new_node("grandchild");
    child.append(grandchild, &mut arena);
    // root
    // `-- child
    //     `-- grandchild
    assert!(matches!(
        grandchild.checked_prepend(root, &mut arena),
        Err(NodeError::PrependAncestor)
    ));
    assert!(matches!(
        grandchild.checked_prepend(child, &mut arena),
        Err(NodeError::PrependAncestor)
    ));
}

#[test]
fn reserve() {
    let mut arena = Arena::new();
    arena.new_node(1);
    arena.reserve(5);
    assert!(arena.capacity() >= 5);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn inaccessible_node() {
    let mut arena = Arena::new();
    let n1_id = arena.new_node("1");
    let n2_id = arena.new_node("2");
    arena.clear();
    assert!(arena.get(n1_id).is_none());
    let n1_id = arena.new_node("1");
    assert_eq!(*arena[n1_id].get(), "1");
    n2_id.is_removed(&arena);
}

#[test]
fn prepend_value() {
    let mut arena = Arena::new();
    let root = arena.new_node(10);
    let c1 = root.prepend_value(1, &mut arena);
    let c2 = root.prepend_value(2, &mut arena);
    let c3 = root.prepend_value(3, &mut arena);
    let children: Vec<_> = root.children(&arena).collect();
    assert_eq!(children, vec![c3, c2, c1]);
}

#[test]
fn reverse_children() {
    let mut arena = Arena::new();
    let root = arena.new_node(10);
    root.append_value(1, &mut arena);
    root.append_value(2, &mut arena);
    root.append_value(3, &mut arena);
    let mut iter = root.children(&arena).rev().map(|n| *arena[n].get());
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);
}

#[test]
fn detach_children_no_children() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let child = arena.new_node("child");
    root.append(child, &mut arena);

    // Detach children of a leaf node (no-op)
    child.detach_children(&mut arena);
    assert_eq!(child.children(&arena).count(), 0);
    // Parent relationship preserved
    assert_eq!(child.parent(&arena), Some(root));
}

#[test]
fn detach_children_single_child() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let child = arena.new_node("child");
    root.append(child, &mut arena);

    root.detach_children(&mut arena);
    assert_eq!(root.children(&arena).count(), 0);
    assert!(arena[child].parent().is_none());
    assert!(!arena[child].is_removed());
}

#[test]
fn detach_children_preserves_parent_position() {
    let mut arena = Arena::new();
    let grandparent = arena.new_node("gp");
    let parent = arena.new_node("p");
    let sibling = arena.new_node("s");
    grandparent.append(parent, &mut arena);
    grandparent.append(sibling, &mut arena);
    let c1 = arena.new_node("c1");
    let c2 = arena.new_node("c2");
    parent.append(c1, &mut arena);
    parent.append(c2, &mut arena);

    parent.detach_children(&mut arena);

    // Parent still in its original position
    assert_eq!(parent.parent(&arena), Some(grandparent));
    assert_eq!(arena[parent].next_sibling(), Some(sibling));
    // Children are detached and independent
    assert!(arena[c1].parent().is_none());
    assert!(arena[c1].next_sibling().is_none());
    assert!(arena[c2].parent().is_none());
    assert!(arena[c2].previous_sibling().is_none());
}

#[test]
fn remove_children_no_children() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let child = arena.new_node("child");
    root.append(child, &mut arena);

    // Remove children of a leaf node (no-op)
    child.remove_children(&mut arena);
    assert_eq!(child.children(&arena).count(), 0);
    assert_eq!(child.parent(&arena), Some(root));
}

#[test]
fn remove_children_preserves_parent_position() {
    let mut arena = Arena::new();
    let grandparent = arena.new_node("gp");
    let parent = arena.new_node("p");
    let sibling = arena.new_node("s");
    grandparent.append(parent, &mut arena);
    grandparent.append(sibling, &mut arena);
    let c1 = arena.new_node("c1");
    let c1_1 = arena.new_node("c1_1");
    let c2 = arena.new_node("c2");
    parent.append(c1, &mut arena);
    c1.append(c1_1, &mut arena);
    parent.append(c2, &mut arena);

    parent.remove_children(&mut arena);

    // Parent still in its original position
    assert_eq!(parent.parent(&arena), Some(grandparent));
    assert_eq!(arena[parent].next_sibling(), Some(sibling));
    assert_eq!(parent.children(&arena).count(), 0);
    // All children and grandchildren removed
    assert!(c1.is_removed(&arena));
    assert!(c1_1.is_removed(&arena));
    assert!(c2.is_removed(&arena));
}

#[test]
fn reverse_traverse() {
    use indextree::NodeEdge;

    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);

    let forward: Vec<_> = n1.traverse(&arena).collect();
    let mut reverse: Vec<_> = n1.reverse_traverse(&arena).collect();
    reverse.reverse();
    assert_eq!(forward, reverse);

    // Verify specific order
    let events: Vec<_> = n1.reverse_traverse(&arena).collect();
    assert_eq!(events[0], NodeEdge::End(n1));
    assert_eq!(events[1], NodeEdge::End(n1_2));
    assert_eq!(events[2], NodeEdge::Start(n1_2));
    assert_eq!(events[3], NodeEdge::End(n1_1));
    assert_eq!(events[4], NodeEdge::Start(n1_1));
    assert_eq!(events[5], NodeEdge::Start(n1));
    assert_eq!(events.len(), 6);
}

#[test]
fn predecessors() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);

    let preds: Vec<_> = n1_2.predecessors(&arena).collect();
    assert_eq!(preds, vec![n1_2, n1_1, n1]);
}

#[test]
fn iter_node_ids() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n2 = arena.new_node("2");
    let n3 = arena.new_node("3");
    n2.remove(&mut arena);

    let ids: Vec<_> = arena.iter_node_ids().collect();
    assert_eq!(ids, vec![n1, n3]);
}

#[test]
fn node_display() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n2 = arena.new_node("2");
    n1.append(n2, &mut arena);

    let display = format!("{}", arena[n1]);
    assert!(display.contains("no parent"));
    assert!(display.contains("first child"));

    let display = format!("{}", arena[n2]);
    assert!(display.contains("parent:"));
    assert!(display.contains("no first child"));
}

#[test]
fn node_error_display() {
    let err = NodeError::AppendSelf;
    assert_eq!(format!("{err}"), "Can not append a node to itself");

    let err = NodeError::Removed;
    assert_eq!(
        format!("{err}"),
        "Removed node cannot have any parent, siblings, and children"
    );
}

#[test]
fn last_child_accessor() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert_eq!(arena[n1].last_child(), None);

    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);

    assert_eq!(arena[n1].last_child(), Some(n1_2));
}

#[test]
fn children_reverse_iterator() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let c1 = arena.new_node("c1");
    let c2 = arena.new_node("c2");
    let c3 = arena.new_node("c3");
    root.append(c1, &mut arena);
    root.append(c2, &mut arena);
    root.append(c3, &mut arena);

    let forward: Vec<_> = root.children(&arena).collect();
    assert_eq!(forward, vec![c1, c2, c3]);

    let backward: Vec<_> = root.children(&arena).rev().collect();
    assert_eq!(backward, vec![c3, c2, c1]);
}

#[test]
fn following_siblings_reverse() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let c1 = arena.new_node("c1");
    let c2 = arena.new_node("c2");
    let c3 = arena.new_node("c3");
    root.append(c1, &mut arena);
    root.append(c2, &mut arena);
    root.append(c3, &mut arena);

    let forward: Vec<_> = c1.following_siblings(&arena).collect();
    assert_eq!(forward, vec![c1, c2, c3]);

    let backward: Vec<_> = c1.following_siblings(&arena).rev().collect();
    assert_eq!(backward, vec![c3, c2, c1]);
}

#[test]
fn preceding_siblings_reverse() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let c1 = arena.new_node("c1");
    let c2 = arena.new_node("c2");
    let c3 = arena.new_node("c3");
    root.append(c1, &mut arena);
    root.append(c2, &mut arena);
    root.append(c3, &mut arena);

    let forward: Vec<_> = c3.preceding_siblings(&arena).collect();
    assert_eq!(forward, vec![c3, c2, c1]);

    let backward: Vec<_> = c3.preceding_siblings(&arena).rev().collect();
    assert_eq!(backward, vec![c1, c2, c3]);
}

#[test]
fn node_error_display_all_variants() {
    let err = NodeError::PrependSelf;
    assert_eq!(format!("{err}"), "Can not prepend a node to itself");

    let err = NodeError::InsertBeforeSelf;
    assert_eq!(format!("{err}"), "Can not insert a node before itself");

    let err = NodeError::InsertAfterSelf;
    assert_eq!(format!("{err}"), "Can not insert a node after itself");

    let err = NodeError::AppendAncestor;
    assert_eq!(format!("{err}"), "Can not append a node to its descendant");

    let err = NodeError::PrependAncestor;
    assert_eq!(format!("{err}"), "Can not prepend a node to its descendant");
}

#[test]
fn panicking_wrappers() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let c1 = arena.new_node("c1");
    let c2 = arena.new_node("c2");
    let c3 = arena.new_node("c3");

    root.prepend(c1, &mut arena);
    root.prepend(c2, &mut arena);
    let children: Vec<_> = root.children(&arena).collect();
    assert_eq!(children, vec![c2, c1]);

    c2.insert_after(c3, &mut arena);
    let children: Vec<_> = root.children(&arena).collect();
    assert_eq!(children, vec![c2, c3, c1]);

    let c4 = arena.new_node("c4");
    c3.insert_before(c4, &mut arena);
    let children: Vec<_> = root.children(&arena).collect();
    assert_eq!(children, vec![c2, c4, c3, c1]);
}

#[test]
fn get_node_id_at() {
    use std::num::NonZeroUsize;

    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n2 = arena.new_node("2");
    let n3 = arena.new_node("3");

    let idx1: NonZeroUsize = n1.into();
    let idx2: NonZeroUsize = n2.into();
    let idx3: NonZeroUsize = n3.into();

    assert_eq!(arena.get_node_id_at(idx1), Some(n1));
    assert_eq!(arena.get_node_id_at(idx2), Some(n2));
    assert_eq!(arena.get_node_id_at(idx3), Some(n3));

    // Out of bounds index returns None
    let out_of_bounds = NonZeroUsize::new(100).unwrap();
    assert_eq!(arena.get_node_id_at(out_of_bounds), None);

    // Removed node returns None
    n2.remove(&mut arena);
    assert_eq!(arena.get_node_id_at(idx2), None);
}

#[test]
fn as_slice() {
    let mut arena = Arena::new();
    let n1 = arena.new_node(10);
    let n2 = arena.new_node(20);
    let n3 = arena.new_node(30);
    n1.append(n2, &mut arena);
    n1.append(n3, &mut arena);

    let slice = arena.as_slice();
    assert_eq!(slice.len(), 3);
    assert_eq!(*slice[0].get(), 10);
    assert_eq!(*slice[1].get(), 20);
    assert_eq!(*slice[2].get(), 30);
}
