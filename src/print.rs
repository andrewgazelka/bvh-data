use std::alloc::Allocator;
use std::collections::VecDeque;

use crate::node::Expanded;
use crate::Bvh;

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
        queue.push_back((self.root_context(), 0));

        while let Some((context, depth)) = queue.pop_back() {
            let indent = "  ".repeat(depth);
            let idx = context.idx;
            let node = unsafe { self.nodes[idx as usize].assume_init_ref().get() };


            match node.into_expanded() {
                Expanded::Aabb(aabb) => {
                    output.push_str(&format!("{idx:02}\t{indent}Internal({aabb:?})\n"));

                    let left = context.left();
                    let right = context.right();

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
