use indextree::{Arena, macros::tree};

fn main() {
    let mut arena = Arena::new();
    let root_node = arena.new_node("my root node");

    tree! {
        &mut arena,
        root_node,
        "1",
        "2" => { "2_1" },
        "3"
    };

    println!("{}", root_node.debug_pretty_print(&arena));
}
