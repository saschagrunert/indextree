use std::fmt::{Debug, Display};

use indextree::{Arena, NodeId};
use indextree_macros::tree;

pub fn compare_nodes<T>(arena: &Arena<T>, n1: NodeId, n2: NodeId)
where
    T: Debug + Display + Clone + PartialEq,
{
    let get_val = |id: NodeId| -> T { arena.get(id).unwrap().get().clone() };

    let n1_iter = n1.descendants(arena).skip(1).map(get_val);
    let n2_iter = n2.descendants(arena).skip(1).map(get_val);

    assert!(
        Iterator::eq(n1_iter, n2_iter),
        r#"Tree equality assertion failed!

### Left Tree ###

{}

### Right Tree ###

{}"#,
        n1.debug_pretty_print(arena),
        n2.debug_pretty_print(arena),
    );
}

#[test]
fn no_nesting() {
    let mut arena = Arena::new();

    let root_macro = tree! {
        &mut arena,
        "macro root node" => {
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            "10",
        }
    };

    let root_proc = arena.new_node("procedural root node");
    root_proc.append_value("1", &mut arena);
    root_proc.append_value("2", &mut arena);
    root_proc.append_value("3", &mut arena);
    root_proc.append_value("4", &mut arena);
    root_proc.append_value("5", &mut arena);
    root_proc.append_value("6", &mut arena);
    root_proc.append_value("7", &mut arena);
    root_proc.append_value("8", &mut arena);
    root_proc.append_value("9", &mut arena);
    root_proc.append_value("10", &mut arena);

    compare_nodes(&arena, root_proc, root_macro);
}

#[test]
fn mild_nesting() {
    let mut arena = Arena::new();

    let root_macro = arena.new_node("macro root node");
    tree!(
        &mut arena,
        root_macro => {
            "1",
            "2" => {
                "2_1" => { "2_1_1" },
                "2_2",
            },
            "3",
        }
    );

    let root_proc = arena.new_node("proc root node");
    root_proc.append_value("1", &mut arena);
    let node_2 = root_proc.append_value("2", &mut arena);
    let node_2_1 = node_2.append_value("2_1", &mut arena);
    node_2_1.append_value("2_1_1", &mut arena);
    node_2.append_value("2_2", &mut arena);
    root_proc.append_value("3", &mut arena);

    compare_nodes(&arena, root_proc, root_macro);
}
