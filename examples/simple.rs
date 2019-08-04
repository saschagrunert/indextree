use indextree::Arena;

pub fn main() {
    // Create a new arena
    let arena = &mut Arena::new();

    // Add some new nodes to the arena
    let a = arena.new_node(1);
    let b = arena.new_node(2);

    // Append a to b
    a.append(b, arena);
    assert_eq!(b.ancestors(arena).into_iter().count(), 2);
}
