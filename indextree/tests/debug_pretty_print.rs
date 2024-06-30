//! Tests for debug pretty printing.

use core::fmt;

use indextree::{Arena, NodeId};

#[derive(Clone)]
struct Label(Vec<i32>);

impl fmt::Debug for Label {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use `Vec<i32>`'s debug formatting.
        self.0.fmt(f)
    }
}

impl fmt::Display for Label {
    // `1/2/3` for normal formatting, and `1 -> 2 -> 3` for alternate formatting.
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();
        match iter.next() {
            Some(v) => write!(f, "{}", v)?,
            None => return write!(f, "root"),
        }
        if f.alternate() {
            iter.try_for_each(|v| write!(f, " -> {}", v))
        } else {
            iter.try_for_each(|v| write!(f, "/{}", v))
        }
    }
}

macro_rules! label {
    ($($tt:tt)*) => {
        Label(vec![$($tt)*])
    }
}

/// Returns the sample tree and the root node ID.
fn sample_tree() -> (Arena<Label>, NodeId) {
    //  []
    //  |-- [0]
    //  |-- [1]
    //  |   |-- [1, 0]
    //  |   |   `-- [1, 0, 0]
    //  |   |-- [1, 1]
    //  |   `-- [1, 2]
    //  |       `-- [1, 2, 0]
    //  `-- [2]
    //      |-- [2, 0]
    //      |   `-- [2, 0, 0]
    //      `-- [2, 1]
    //          `-- [2, 1, 0]
    let mut arena = Arena::new();
    let root = arena.new_node(label![]);
    let n0 = arena.new_node(label![0]);
    root.append(n0, &mut arena);
    let n1 = arena.new_node(label![1]);
    root.append(n1, &mut arena);
    let n1_0 = arena.new_node(label![1, 0]);
    n1.append(n1_0, &mut arena);
    let n1_0_0 = arena.new_node(label![1, 0, 0]);
    n1_0.append(n1_0_0, &mut arena);
    let n1_1 = arena.new_node(label![1, 1]);
    n1.append(n1_1, &mut arena);
    let n1_2 = arena.new_node(label![1, 2]);
    n1.append(n1_2, &mut arena);
    let n1_2_0 = arena.new_node(label![1, 2, 0]);
    n1_2.append(n1_2_0, &mut arena);
    let n2 = arena.new_node(label![2]);
    root.append(n2, &mut arena);
    let n2_0 = arena.new_node(label![2, 0]);
    n2.append(n2_0, &mut arena);
    let n2_0_0 = arena.new_node(label![2, 0, 0]);
    n2_0.append(n2_0_0, &mut arena);
    let n2_1 = arena.new_node(label![2, 1]);
    n2.append(n2_1, &mut arena);
    let n2_1_0 = arena.new_node(label![2, 1, 0]);
    n2_1.append(n2_1_0, &mut arena);

    (arena, root)
}

#[test]
fn debug() {
    const EXPECTED: &str = r#"[]
|-- [0]
|-- [1]
|   |-- [1, 0]
|   |   `-- [1, 0, 0]
|   |-- [1, 1]
|   `-- [1, 2]
|       `-- [1, 2, 0]
`-- [2]
    |-- [2, 0]
    |   `-- [2, 0, 0]
    `-- [2, 1]
        `-- [2, 1, 0]"#;

    let (arena, root) = sample_tree();
    assert_eq!(format!("{:?}", root.debug_pretty_print(&arena)), EXPECTED);
}

#[test]
fn debug_alternate() {
    const EXPECTED: &str = r#"[]
|-- [
|       0,
|   ]
|-- [
|       1,
|   ]
|   |-- [
|   |       1,
|   |       0,
|   |   ]
|   |   `-- [
|   |           1,
|   |           0,
|   |           0,
|   |       ]
|   |-- [
|   |       1,
|   |       1,
|   |   ]
|   `-- [
|           1,
|           2,
|       ]
|       `-- [
|               1,
|               2,
|               0,
|           ]
`-- [
        2,
    ]
    |-- [
    |       2,
    |       0,
    |   ]
    |   `-- [
    |           2,
    |           0,
    |           0,
    |       ]
    `-- [
            2,
            1,
        ]
        `-- [
                2,
                1,
                0,
            ]"#;

    let (arena, root) = sample_tree();
    assert_eq!(format!("{:#?}", root.debug_pretty_print(&arena)), EXPECTED);
}

#[test]
fn display() {
    const EXPECTED: &str = r#"root
|-- 0
|-- 1
|   |-- 1/0
|   |   `-- 1/0/0
|   |-- 1/1
|   `-- 1/2
|       `-- 1/2/0
`-- 2
    |-- 2/0
    |   `-- 2/0/0
    `-- 2/1
        `-- 2/1/0"#;

    let (arena, root) = sample_tree();
    assert_eq!(format!("{}", root.debug_pretty_print(&arena)), EXPECTED);
}

#[test]
fn display_alternate() {
    const EXPECTED: &str = r#"root
|-- 0
|-- 1
|   |-- 1 -> 0
|   |   `-- 1 -> 0 -> 0
|   |-- 1 -> 1
|   `-- 1 -> 2
|       `-- 1 -> 2 -> 0
`-- 2
    |-- 2 -> 0
    |   `-- 2 -> 0 -> 0
    `-- 2 -> 1
        `-- 2 -> 1 -> 0"#;

    let (arena, root) = sample_tree();
    assert_eq!(format!("{:#}", root.debug_pretty_print(&arena)), EXPECTED);
}

#[test]
fn non_debug_printable_type() {
    #[derive(Clone)]
    struct NonDebug(i32);
    impl fmt::Display for NonDebug {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    let mut arena = Arena::new();
    let root = arena.new_node(NonDebug(0));
    let child = arena.new_node(NonDebug(1));
    root.append(child, &mut arena);

    const EXPECTED: &str = "0\n`-- 1";
    assert_eq!(root.debug_pretty_print(&arena).to_string(), EXPECTED);
}
