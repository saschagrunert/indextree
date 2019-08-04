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
