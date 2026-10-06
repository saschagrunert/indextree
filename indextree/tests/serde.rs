#![cfg(feature = "serde")]

use indextree::Arena;
use serde_json::json;

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

#[test]
fn deserialize_rejects_broken_parent_pointer() {
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": {"index1": 2, "stamp": 0},
                "last_child": {"index1": 2, "stamp": 0},
                "stamp": 0,
                "data": {"Data": 1}
            },
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": 0,
                "data": {"Data": 2}
            }
        ],
        "first_free_slot": null,
        "last_free_slot": null
    });
    let err = serde_json::from_value::<Arena<i32>>(val).unwrap_err();
    assert!(err.to_string().starts_with("invalid arena"), "{err}");
}

#[test]
fn deserialize_rejects_live_node_with_next_free_data() {
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": 0,
                "data": {"NextFree": null}
            }
        ],
        "first_free_slot": null,
        "last_free_slot": null
    });
    let err = serde_json::from_value::<Arena<i32>>(val).unwrap_err();
    assert!(err.to_string().starts_with("invalid arena"), "{err}");
}

#[test]
fn deserialize_rejects_broken_sibling_chain() {
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": {"index1": 2, "stamp": 0},
                "last_child": {"index1": 3, "stamp": 0},
                "stamp": 0,
                "data": {"Data": 1}
            },
            {
                "parent": {"index1": 1, "stamp": 0},
                "previous_sibling": null,
                "next_sibling": {"index1": 3, "stamp": 0},
                "first_child": null,
                "last_child": null,
                "stamp": 0,
                "data": {"Data": 2}
            },
            {
                "parent": {"index1": 1, "stamp": 0},
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": 0,
                "data": {"Data": 3}
            }
        ],
        "first_free_slot": null,
        "last_free_slot": null
    });
    let err = serde_json::from_value::<Arena<i32>>(val).unwrap_err();
    assert!(err.to_string().starts_with("invalid arena"), "{err}");
}

#[test]
fn deserialize_rejects_mismatched_free_list_pointers() {
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": 0,
                "data": {"Data": 1}
            }
        ],
        "first_free_slot": 0,
        "last_free_slot": null
    });
    let err = serde_json::from_value::<Arena<i32>>(val).unwrap_err();
    assert!(err.to_string().starts_with("invalid arena"), "{err}");
}

#[test]
fn deserialize_rejects_stale_stamp_in_child_pointer() {
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": {"index1": 2, "stamp": 99},
                "last_child": {"index1": 2, "stamp": 99},
                "stamp": 0,
                "data": {"Data": 1}
            },
            {
                "parent": {"index1": 1, "stamp": 0},
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": 0,
                "data": {"Data": 2}
            }
        ],
        "first_free_slot": null,
        "last_free_slot": null
    });
    let err = serde_json::from_value::<Arena<i32>>(val).unwrap_err();
    assert!(err.to_string().starts_with("invalid arena"), "{err}");
}

#[test]
fn deserialize_rejects_out_of_bounds_pointer() {
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": {"index1": 99, "stamp": 0},
                "last_child": {"index1": 99, "stamp": 0},
                "stamp": 0,
                "data": {"Data": 1}
            }
        ],
        "first_free_slot": null,
        "last_free_slot": null
    });
    let err = serde_json::from_value::<Arena<i32>>(val).unwrap_err();
    assert!(err.to_string().starts_with("invalid arena"), "{err}");
}

#[test]
fn deserialize_restores_live_count() {
    let mut arena = Arena::new();
    let root = arena.new_node(1);
    let child = root.append_value(2, &mut arena);
    root.append_value(3, &mut arena);
    child.remove(&mut arena);

    let json = serde_json::to_value(&arena).unwrap();
    assert!(json.get("live").is_none());
    let mut deserialized: Arena<i32> = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.live_count(), 2);
    assert_eq!(deserialized, arena);

    deserialized.new_node(4);
    assert_eq!(deserialized.live_count(), 3);
}

#[test]
fn deserialize_unlinks_exhausted_free_slots() {
    // Older versions kept a slot with stamp -32767 in the free list, which
    // can no longer be reused.
    let val = json!({
        "nodes": [
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": -1,
                "data": {"NextFree": 1}
            },
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": -32767,
                "data": {"NextFree": 2}
            },
            {
                "parent": null,
                "previous_sibling": null,
                "next_sibling": null,
                "first_child": null,
                "last_child": null,
                "stamp": -3,
                "data": {"NextFree": null}
            }
        ],
        "first_free_slot": 0,
        "last_free_slot": 2
    });
    let mut arena: Arena<i32> = serde_json::from_value(val).unwrap();
    assert!(arena.validate());
    let a = arena.new_node(1);
    let b = arena.new_node(2);
    let c = arena.new_node(3);
    assert_eq!(usize::from(a), 1);
    assert_eq!(usize::from(b), 3);
    // The exhausted slot is skipped, so a new slot gets appended.
    assert_eq!(usize::from(c), 4);
    assert_eq!(arena.live_count(), 3);
}

#[test]
fn debug_output_is_unchanged() {
    let mut arena = Arena::new();
    arena.new_node(1);
    assert!(!format!("{arena:?}").contains("live"));
}

#[test]
fn round_trip_equals_arena_with_drifted_live_count() {
    // Swapping nodes through `IndexMut` is not tracked by the live count,
    // which must not affect equality.
    let mut x = Arena::new();
    let live = x.new_node(1);
    let mut y = Arena::new();
    let removed = y.new_node(2);
    removed.remove(&mut y);
    core::mem::swap(&mut x[live], &mut y[removed]);
    assert_eq!(x.live_count(), 1);

    let json = serde_json::to_string(&x).unwrap();
    let deserialized: Arena<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.live_count(), 0);
    assert_eq!(x, deserialized);
}
