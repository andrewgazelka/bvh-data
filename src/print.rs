use crate::dfs::context::DfsContext;
use crate::Bvh;
use std::alloc::Allocator;

use std::collections::VecDeque;

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

        while let Some((context, depth)) = queue.pop_front() {
            let indent = "  ".repeat(depth);
            let node = unsafe { self.nodes[context.idx as usize].assume_init_ref().get() };

            // leaf
            if let Some(idx) = node.leaf_element_indices() {
                output.push_str(&format!("{}Leaf(data_index: {})\n", indent, idx));
                continue;
            }

            output.push_str(&format!("{}Internal\n", indent));

            let left = context.left();
            let right = context.right();

            queue.push_back((left, depth + 1));
            queue.push_back((right, depth + 1));
        }
    }
}
