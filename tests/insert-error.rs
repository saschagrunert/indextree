//! Insertion errors.

use indextree::Arena;

#[test]
fn append_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.append(n1, &mut arena).is_err());
}

#[test]
fn prepend_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.prepend(n1, &mut arena).is_err());
}

#[test]
fn insert_after_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.insert_after(n1, &mut arena).is_err());
}

#[test]
fn insert_before_self() {
    let mut arena = Arena::new();
    let n1 = arena.new_node("1");
    assert!(n1.insert_before(n1, &mut arena).is_err());
}
