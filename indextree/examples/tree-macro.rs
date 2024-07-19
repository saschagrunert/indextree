use indextree::{macros::tree, Arena};

fn main() {
    let mut arena = Arena::new();

    // It works with existing nodes
    let root_node = arena.new_node("my root node");
    tree!(
        &mut arena,
        root_node => {
            "1",
            "2" => {
                "2_1" => { "2_1_1" },
                "2_2",
            },
            "3" => {},
        }
    );

    println!("{}", root_node.debug_pretty_print(&arena));

    // It can also create a root node for you!
    let root_node = tree!(
        &mut arena,
        "my root node, but automagically created" => {
            "1",
            "2" => {
                "2_1" => { "2_1_1" },
                "2_2",
            },
            "3",
        }
    );

    println!("{}", root_node.debug_pretty_print(&arena));
}
