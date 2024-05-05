use crate::aabb::Aabb;
use crate::dfs::context::Dfs;
use crate::node::Expanded;
use crate::Bvh;
use arrayvec::ArrayVec;
use std::alloc::Allocator;
use std::ops::Range;

const MAX_SIZE: usize = 32;
const DFS_STACK_SIZE: usize = 32;

impl<T, A: Allocator> Bvh<T, A> {
    pub fn get_in(&self, query: Aabb) -> impl IntoIterator<Item = Range<usize>> {
        let mut to_send_indices: ArrayVec<Range<usize>, MAX_SIZE> = ArrayVec::new();
        let mut dfs_stack: ArrayVec<Dfs, DFS_STACK_SIZE> = ArrayVec::new();

        // so we do not need special case (there is always a last)
        to_send_indices.push(0..0);
        dfs_stack.push(self.root_context());

        while let Some(context) = dfs_stack.pop() {
            let node = unsafe { self.get_node(context.idx as usize) };

            match node.into_expanded() {
                Expanded::Leaf(leaf) => {
                    if !query.contains_point(leaf.point) {
                        continue;
                    }

                    let start = leaf.start;
                    let end = self.get_next_data_for_idx(context.idx);

                    let last = unsafe { to_send_indices.last_mut().unwrap_unchecked() };

                    if last.end == start as usize {
                        // combine
                        last.end = end;
                    } else {
                        to_send_indices.push(start as usize..end);
                    }
                }
                Expanded::Aabb(aabb) => {
                    if !aabb.intersects(query) {
                        continue;
                    }

                    let left = context.left();
                    let right = context.right();

                    dfs_stack.push(right);

                    // we want to do left first because this is how we are doing DFS when building the tree
                    // if we do not do this in the right order dfs_stack will be in the wrong order
                    dfs_stack.push(left);
                }
            }
        }

        // todo: we should probably remove 0..0 if it still exists

        to_send_indices
    }
}
