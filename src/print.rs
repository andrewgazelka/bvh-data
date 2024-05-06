use std::alloc::Allocator;
use std::collections::VecDeque;

use crate::node::Expanded;
use crate::{child_left, child_right, Bvh, ROOT_IDX};

impl<T, A: Allocator> Bvh<T, A> {
    pub fn print(&self) -> String {
        let mut output = String::new();
        self.print_helper(&mut output);

        // trim last newline
        if output.ends_with('\n') {
            output.pop();
        }

        output
    }

    fn print_helper(&self, output: &mut String) {
        let mut queue = VecDeque::new();
        queue.push_back((ROOT_IDX, 0));

        while let Some((idx, depth)) = queue.pop_back() {
            let indent = "  ".repeat(depth);
            let node = unsafe { self.get_node(idx) };

            match node.into_expanded() {
                Expanded::Aabb(aabb) => {
                    output.push_str(&format!("{idx:02}\t{indent}Internal({aabb:?})\n"));

                    let left = child_left(idx);
                    let right = child_right(idx);

                    queue.push_back((left, depth + 1));
                    queue.push_back((right, depth + 1));
                }
                Expanded::Leaf(leaf) => {
                    output.push_str(&format!("{idx:02}\t{indent}Leaf({leaf})\n"));
                }
            }
        }
    }
}
