//! Applies long random sequences of tree operations, including operations on
//! removed and stale node IDs, and checks the arena consistency after each.

use indextree::{Arena, NodeId};

/// Small xorshift PRNG to keep the test deterministic and dependency free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn check(arena: &Arena<usize>, step: usize) {
    assert!(arena.validate(), "arena invalid after step {step}");

    // Every traversal must terminate, which would not be the case for cycles.
    let live = arena.iter().filter(|n| !n.is_removed()).count();
    let reachable: usize = arena
        .iter_node_ids()
        .filter(|&id| id.is_root(arena))
        .map(|root| root.descendants(arena).take(live + 1).count())
        .sum();
    assert_eq!(reachable, live, "unreachable or cyclic nodes after {step}");
}

fn run(seed: u64, steps: usize) {
    let mut rng = Rng(seed);
    let mut arena = Arena::new();
    // All IDs ever created, so that removed and stale IDs get used as well.
    let mut ids: Vec<NodeId> = Vec::new();

    for step in 0..steps {
        if ids.is_empty() || rng.below(8) == 0 {
            ids.push(arena.new_node(step));
            continue;
        }
        let a = ids[rng.below(ids.len())];
        let b = ids[rng.below(ids.len())];
        let a_removed = a.is_removed(&arena);
        let before = arena.clone();
        // Whether the operation must leave the arena unchanged, i.e. it
        // failed or was called with a removed (possibly stale) node ID.
        let unchanged = match rng.below(14) {
            0 => a.checked_append(b, &mut arena).is_err(),
            1 => a.checked_prepend(b, &mut arena).is_err(),
            2 => a.checked_insert_after(b, &mut arena).is_err(),
            3 => a.checked_insert_before(b, &mut arena).is_err(),
            4 if !a_removed => {
                ids.push(a.append_value(step, &mut arena));
                false
            }
            5 if !a_removed => {
                ids.push(a.prepend_value(step, &mut arena));
                false
            }
            6 if !a_removed => {
                ids.push(a.insert_after_value(step, &mut arena));
                false
            }
            7 if !a_removed => {
                ids.push(a.insert_before_value(step, &mut arena));
                false
            }
            8 => {
                a.remove(&mut arena);
                a_removed
            }
            9 if rng.below(4) == 0 => {
                a.remove_subtree(&mut arena);
                a_removed
            }
            10 if rng.below(4) == 0 => {
                a.remove_children(&mut arena);
                a_removed
            }
            11 => {
                a.detach(&mut arena);
                a_removed
            }
            12 => {
                a.detach_children(&mut arena);
                a_removed
            }
            13 => a.checked_reparent(b, &mut arena).is_err(),
            _ => true,
        };
        if unchanged {
            assert!(arena == before, "arena changed by no-op at step {step}");
        }
        check(&arena, step);
    }
}

#[test]
fn random_operations() {
    for seed in 1..=32u64 {
        run(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), 2_000);
    }
}
