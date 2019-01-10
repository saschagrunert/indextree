use indextree::Arena;

pub fn main() {
    // Create a new arena
    let arena = &mut Arena::new();

    // Add some new nodes to the arena
    let a = arena.new_node(1);
    let b = arena.new_node(2);

    // Append a to b
    assert!(a.append(b, arena).is_ok());
    assert_eq!(b.ancestors(arena).into_iter().count(), 2);
}
