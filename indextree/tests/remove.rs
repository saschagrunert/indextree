use indextree::{
    Arena,
    NodeEdge::{End, Start},
};

#[test]
fn toplevel_with_no_child() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    // arena
    // `-- 1
    n1.remove(&mut arena);
}

#[test]
fn toplevel_with_single_child() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    // arena
    // `-- 1 *
    //     `-- 1_1
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[Start(n1), Start(n1_1), End(n1_1), End(n1)]
    );
    n1.remove(&mut arena);
    // arena
    // `-- 1_1
    assert!(arena[n1_1].parent().is_none());
}

#[test]
fn toplevel_with_multiple_children() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);
    // arena
    // `-- 1 *
    //     |-- 1_1
    //     `-- 1_2
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            End(n1_2),
            End(n1)
        ]
    );
    n1.remove(&mut arena);
    // arena
    // |-- 1_1
    // `-- 1_2
    assert!(arena[n1_1].parent().is_none());
    assert!(arena[n1_2].parent().is_none());
    assert_eq!(
        n1_1.following_siblings(&arena).collect::<Vec<_>>(),
        &[n1_1, n1_2]
    );
}

#[test]
fn single_child_with_no_children() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    // arena
    // `-- 1
    //     `-- 1_1 *
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[Start(n1), Start(n1_1), End(n1_1), End(n1),]
    );
    n1_1.remove(&mut arena);
    // arena
    // `-- 1
    assert!(arena[n1_1].parent().is_none());
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[Start(n1), End(n1),]
    );
}

#[test]
fn single_child_with_single_child() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_1_1 = arena.new_node("1_1_1");
    n1_1.append(n1_1_1, &mut arena);
    // arena
    // `-- 1
    //     `-- 1_1 *
    //         `-- 1_1_1
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            Start(n1_1_1),
            End(n1_1_1),
            End(n1_1),
            End(n1),
        ]
    );
    n1_1.remove(&mut arena);
    // arena
    // `-- 1
    //     `-- 1_1_1
    assert!(arena[n1_1].parent().is_none());
    assert!(arena[n1_1].first_child().is_none());
    assert_eq!(n1_1_1.ancestors(&arena).collect::<Vec<_>>(), &[n1_1_1, n1]);
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[Start(n1), Start(n1_1_1), End(n1_1_1), End(n1),]
    );
}

#[test]
fn first_child_with_no_children() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);
    let n1_3 = arena.new_node("1_3");
    n1.append(n1_3, &mut arena);
    // arena
    // `-- 1
    //     |-- 1_1 *
    //     |-- 1_2
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            End(n1_2),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
    n1_1.remove(&mut arena);
    // arena
    // `-- 1
    //     |-- 1_2
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_2),
            End(n1_2),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
}

#[test]
fn middle_child_with_no_children() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);
    let n1_3 = arena.new_node("1_3");
    n1.append(n1_3, &mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     |-- 1_2 *
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            End(n1_2),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
    n1_2.remove(&mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
}

#[test]
fn last_child_with_no_children() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);
    let n1_3 = arena.new_node("1_3");
    n1.append(n1_3, &mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     |-- 1_2
    //     `-- 1_3 *
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            End(n1_2),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
    n1_3.remove(&mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     `-- 1_2
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            End(n1_2),
            End(n1),
        ]
    );
}

#[test]
fn middle_child_with_single_child() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);
    let n1_2_1 = arena.new_node("1_2_1");
    n1_2.append(n1_2_1, &mut arena);
    let n1_3 = arena.new_node("1_3");
    n1.append(n1_3, &mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     |-- 1_2 *
    //     |   `-- 1_2_1
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            Start(n1_2_1),
            End(n1_2_1),
            End(n1_2),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
    n1_2.remove(&mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     |-- 1_2_1
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2_1),
            End(n1_2_1),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
}

#[test]
fn middle_child_with_multiple_children() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    let n1_1 = arena.new_node("1_1");
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node("1_2");
    n1.append(n1_2, &mut arena);
    let n1_2_1 = arena.new_node("1_2_1");
    n1_2.append(n1_2_1, &mut arena);
    let n1_2_2 = arena.new_node("1_2_2");
    n1_2.append(n1_2_2, &mut arena);
    let n1_2_3 = arena.new_node("1_2_3");
    n1_2.append(n1_2_3, &mut arena);
    let n1_3 = arena.new_node("1_3");
    n1.append(n1_3, &mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     |-- 1_2 *
    //     |   |-- 1_2_1
    //     |   |-- 1_2_2
    //     |   `-- 1_2_3
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2),
            Start(n1_2_1),
            End(n1_2_1),
            Start(n1_2_2),
            End(n1_2_2),
            Start(n1_2_3),
            End(n1_2_3),
            End(n1_2),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
    n1_2.remove(&mut arena);
    // arena
    // `-- 1
    //     |-- 1_1
    //     |-- 1_2_1
    //     |-- 1_2_2
    //     |-- 1_2_3
    //     `-- 1_3
    assert_eq!(
        n1.traverse(&arena).collect::<Vec<_>>(),
        &[
            Start(n1),
            Start(n1_1),
            End(n1_1),
            Start(n1_2_1),
            End(n1_2_1),
            Start(n1_2_2),
            End(n1_2_2),
            Start(n1_2_3),
            End(n1_2_3),
            Start(n1_3),
            End(n1_3),
            End(n1),
        ]
    );
}

#[test]
fn stale_id_does_not_affect_reused_slot() {
    type Op = fn(indextree::NodeId, &mut Arena<&str>);
    let ops: [Op; 5] = [
        |id, arena| id.remove(arena),
        |id, arena| id.remove_subtree(arena),
        |id, arena| id.remove_children(arena),
        |id, arena| id.detach(arena),
        |id, arena| id.detach_children(arena),
    ];
    for op in ops {
        let mut arena = Arena::new();
        let root = arena.new_node("root");
        let stale = arena.new_node("stale");
        stale.remove(&mut arena);
        let new = root.append_value("new", &mut arena);
        // The removed slot got reused.
        assert_eq!(usize::from(stale), usize::from(new));
        new.append_value("child", &mut arena);

        op(stale, &mut arena);

        assert!(!new.is_removed(&arena));
        assert_eq!(new.parent(&arena), Some(root));
        assert_eq!(new.children(&arena).count(), 1);
        assert!(arena.validate());
    }
}

#[test]
fn removal_of_out_of_bounds_id_is_noop() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    arena.clear();
    n1.remove(&mut arena);
    n1.remove_subtree(&mut arena);
    n1.remove_children(&mut arena);
    n1.detach(&mut arena);
    n1.detach_children(&mut arena);
    assert!(arena.is_empty());
}

#[test]
fn removed_descendants_have_no_links() {
    let mut arena = Arena::new();
    let root = arena.new_node("root");
    let n1 = root.append_value("1", &mut arena);
    let n1_1 = n1.append_value("1_1", &mut arena);
    let n1_1_1 = n1_1.append_value("1_1_1", &mut arena);
    let n1_2 = n1.append_value("1_2", &mut arena);
    let n2 = root.append_value("2", &mut arena);
    let n2_1 = n2.append_value("2_1", &mut arena);

    n1.remove_subtree(&mut arena);
    n2.remove_children(&mut arena);

    for id in [n1, n1_1, n1_1_1, n1_2, n2_1] {
        assert!(id.is_removed(&arena));
        let node = &arena[id];
        assert_eq!(node.parent(), None);
        assert_eq!(node.previous_sibling(), None);
        assert_eq!(node.next_sibling(), None);
        assert_eq!(node.first_child(), None);
        assert_eq!(node.last_child(), None);
    }
    assert_eq!(root.children(&arena).collect::<Vec<_>>(), vec![n2]);
    assert!(n2.is_leaf(&arena));
    assert!(arena.validate());
}
