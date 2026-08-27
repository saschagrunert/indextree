#![cfg(feature = "serde")]

use indextree::Arena;

#[test]
fn round_trip_empty_arena() {
    let arena: Arena<i32> = Arena::new();
    let json = serde_json::to_string(&arena).unwrap();
    let deserialized: Arena<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(arena, deserialized);
    assert!(deserialized.validate());
}

#[test]
fn round_trip_single_node() {
    let mut arena = Arena::new();
    let root = arena.new_node("hello");
    let _ = root; // suppress unused warning

    let json = serde_json::to_string(&arena).unwrap();
    let deserialized: Arena<&str> = serde_json::from_str(&json).unwrap();
    assert_eq!(arena, deserialized);
    assert!(deserialized.validate());
}

#[test]
fn round_trip_tree() {
    let mut arena = Arena::new();
    let root = arena.new_node(1);
    let c1 = root.append_value(2, &mut arena);
    let c2 = root.append_value(3, &mut arena);
    c1.append_value(4, &mut arena);
    c2.append_value(5, &mut arena);

    let json = serde_json::to_string(&arena).unwrap();
    let deserialized: Arena<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(arena, deserialized);
    assert!(deserialized.validate());
}

#[test]
fn round_trip_with_removed_node() {
    let mut arena = Arena::new();
    let root = arena.new_node(1);
    let c1 = root.append_value(2, &mut arena);
    root.append_value(3, &mut arena);
    c1.remove(&mut arena);

    let json = serde_json::to_string(&arena).unwrap();
    let deserialized: Arena<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(arena, deserialized);
    assert!(deserialized.validate());
}

#[test]
fn round_trip_with_reused_slot() {
    let mut arena = Arena::new();
    let root = arena.new_node(1);
    let c1 = root.append_value(2, &mut arena);
    root.append_value(3, &mut arena);
    c1.remove(&mut arena);
    let c4 = arena.new_node(4);
    root.append(c4, &mut arena);

    let json = serde_json::to_string(&arena).unwrap();
    let deserialized: Arena<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(arena, deserialized);
    assert!(deserialized.validate());
}
