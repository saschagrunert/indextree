use indextree::*;
use rayon::prelude::*;

pub fn main() {
    // Create a new arena
    let arena = &mut Arena::new();

    // Add some new nodes to the arena
    println!("Creating arena tree");
    let mut last_node = arena.new_node(1);
    for i in 1..10_000_000 {
        let node = arena.new_node(i);
        node.append(last_node, arena);
        last_node = node;
    }

    println!("Parallel iteration over arena tree");
    let _: Vec<f64> = arena
        .par_iter()
        .map(|ref mut i| (*i.get() as f64).sqrt())
        .collect();
}
