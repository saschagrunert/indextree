extern crate indextree;
#[cfg(feature = "par_iter")]
extern crate rayon;

use indextree::Arena;
#[cfg(feature = "par_iter")]
use rayon::prelude::*;

#[test]
fn arenatree_success_create() {
    let mut new_counter = 0;
    let arena = &mut Arena::new();
    macro_rules! new {
        () => {{
            new_counter += 1;
            arena.new_node(new_counter)
        }};
    };

    let a = new!(); // 1
    a.append(new!(), arena); // 2
    a.append(new!(), arena); // 3
    a.prepend(new!(), arena); // 4
    let b = new!(); // 5
    b.append(a, arena);
    a.insert_before(new!(), arena); // 6
    a.insert_before(new!(), arena); // 7
    a.insert_after(new!(), arena); // 8
    a.insert_after(new!(), arena); // 9
    let c = new!(); // 10
    b.append(c, arena);

    arena[c].previous_sibling().unwrap().detach(arena);

    assert_eq!(
        b.descendants(arena).map(|node| arena[node].data).collect::<Vec<_>>(),
        [5, 6, 7, 1, 4, 2, 3, 9, 10]
    );
}

#[test]
#[should_panic]
fn arenatree_failure_prepend() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    a.prepend(b, arena);
}

#[test]
fn arenatree_success_detach() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(1);
    a.append(b, arena);
    assert_eq!(b.ancestors(arena).into_iter().count(), 2);
    b.detach(arena);
    assert_eq!(b.ancestors(arena).into_iter().count(), 1);
}

#[test]
fn arenatree_get() {
    let arena = &mut Arena::new();
    let id = arena.new_node(1);
    assert_eq!(arena.get(id).unwrap().data, 1);
}

#[test]
fn arenatree_get_mut() {
    let arena = &mut Arena::new();
    let id = arena.new_node(1);
    assert_eq!(arena.get_mut(id).unwrap().data, 1);
}

#[test]
fn arenatree_iter() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    a.append(b, arena);
    b.append(c, arena);
    a.append(d, arena);

    let node_refs = arena.iter().collect::<Vec<_>>();
    assert_eq!(node_refs, vec![&arena[a], &arena[b], &arena[c], &arena[d]]);
}

#[cfg(feature = "par_iter")]
#[test]
fn arenatree_par_iter() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    a.append(b, arena);
    b.append(c, arena);
    a.append(d, arena);

    let node_refs = arena.par_iter().collect::<Vec<_>>();
    assert_eq!(node_refs, vec![&arena[a], &arena[b], &arena[c], &arena[d]]);
}
