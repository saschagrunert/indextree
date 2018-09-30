extern crate indextree;
#[cfg(feature = "par_iter")]
extern crate rayon;

use indextree::Arena;
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
    };

    let a = new!(); // 1
    assert!(a.append(new!(), arena).is_ok()); // 2
    assert!(a.append(new!(), arena).is_ok()); // 3
    assert!(a.prepend(new!(), arena).is_ok()); // 4
    let b = new!(); // 5
    assert!(b.append(a, arena).is_ok());
    assert!(a.insert_before(new!(), arena).is_ok()); // 6
    assert!(a.insert_before(new!(), arena).is_ok()); // 7
    assert!(a.insert_after(new!(), arena).is_ok()); // 8
    assert!(a.insert_after(new!(), arena).is_ok()); // 9
    let c = new!(); // 10
    assert!(b.append(c, arena).is_ok());

    arena[c].previous_sibling().unwrap().detach(arena);

    assert_eq!(
        b.descendants(arena)
            .map(|node| arena[node].data)
            .collect::<Vec<_>>(),
        [5, 6, 7, 1, 4, 2, 3, 9, 10]
    );
}

#[test]
fn failure_prepend() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    assert!(!a.prepend(b, arena).is_ok());
}

#[test]
fn success_detach() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(1);
    assert!(a.append(b, arena).is_ok());
    assert_eq!(b.ancestors(arena).into_iter().count(), 2);
    b.detach(arena);
    assert_eq!(b.ancestors(arena).into_iter().count(), 1);
}

#[test]
fn get() {
    let arena = &mut Arena::new();
    let id = arena.new_node(1);
    assert_eq!(arena.get(id).unwrap().data, 1);
}

#[test]
fn get_mut() {
    let arena = &mut Arena::new();
    let id = arena.new_node(1);
    assert_eq!(arena.get_mut(id).unwrap().data, 1);
}

#[test]
fn iter() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    assert!(a.append(b, arena).is_ok());
    assert!(b.append(c, arena).is_ok());
    assert!(a.append(d, arena).is_ok());

    let node_refs = arena.iter().collect::<Vec<_>>();
    assert_eq!(node_refs, vec![&arena[a], &arena[b], &arena[c], &arena[d]]);
}

#[cfg(feature = "par_iter")]
#[test]
fn par_iter() {
    let arena = &mut Arena::new();
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    let d = arena.new_node(4);
    assert!(a.append(b, arena).is_ok());
    assert!(b.append(c, arena).is_ok());
    assert!(a.append(d, arena).is_ok());

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
    assert!(n0.append(n1, arena).is_ok());
    assert!(n0.append(n2, arena).is_ok());
    assert!(n0.append(n3, arena).is_ok());
    assert!(n2.append(n4, arena).is_ok());
    assert!(n2.append(n5, arena).is_ok());
    assert!(n2.append(n5, arena).is_ok());
    assert!(n2.append(n6, arena).is_ok());
    assert!(n2.remove(arena).is_ok());

    let node_refs = arena
        .iter()
        .filter_map(|x| if !x.is_removed() { Some(x.data) } else { None })
        .collect::<Vec<_>>();
    assert_eq!(node_refs, vec![0, 1, 3, 4, 5, 6]);
    assert_eq!(n2.children(arena).collect::<Vec<_>>().len(), 0);
    assert_eq!(n2.descendants(arena).collect::<Vec<_>>().len(), 1);
    assert_eq!(n2.preceding_siblings(arena).collect::<Vec<_>>().len(), 1);
    assert_eq!(n2.following_siblings(arena).collect::<Vec<_>>().len(), 1);

    assert!(n3.remove(arena).is_ok());

    let node_refs = arena
        .iter()
        .filter_map(|x| if !x.is_removed() { Some(x.data) } else { None })
        .collect::<Vec<_>>();
    assert_eq!(node_refs, vec![0, 1, 4, 5, 6]);
    assert_eq!(n3.children(arena).collect::<Vec<_>>().len(), 0);
    assert_eq!(n3.descendants(arena).collect::<Vec<_>>().len(), 1);
    assert_eq!(n3.preceding_siblings(arena).collect::<Vec<_>>().len(), 1);
    assert_eq!(n3.following_siblings(arena).collect::<Vec<_>>().len(), 1);
}
