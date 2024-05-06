use std::alloc::Allocator;
use std::ops::Range;

use arrayvec::ArrayVec;
use glam::I16Vec2;
use heapless::binary_heap::Min;

use crate::aabb::Aabb;
use crate::dfs::context::Dfs;
use crate::node::Expanded;
use crate::Bvh;

const MAX_SIZE: usize = 32;
const DFS_STACK_SIZE: usize = 32;
const HEAP_SIZE: usize = 32;

impl<T, A: Allocator> Bvh<T, A> {
    pub fn get_closest_slice(&self, input: I16Vec2) -> Option<&[T]> {
        let idx = self.get_closest(input)?;
        Some(&self.data[idx])
    }

    /// # Panics
    /// If there are too many elements that overflow `HEAP_SIZE`
    pub fn get_closest(&self, input: I16Vec2) -> Option<Range<usize>> {
        #[derive(Debug, Copy, Clone)]
        struct MinNode {
            dist2: u32,
            expanded: Expanded,
            value: Dfs,
        }

        impl PartialEq for MinNode {
            fn eq(&self, other: &Self) -> bool {
                self.dist2 == other.dist2
            }
        }

        impl Eq for MinNode {}

        impl Ord for MinNode {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.dist2.cmp(&other.dist2)
            }
        }

        impl PartialOrd for MinNode {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut max_distance_to_closest = u32::MAX;

        let mut new_node = |value: Dfs, expanded: Expanded| match expanded {
            Expanded::Aabb(aabb) => {
                let (dist2_min, dist2_max) = aabb.min_max_distance2(input);

                if max_distance_to_closest < dist2_min {
                    return None;
                }

                if dist2_max < max_distance_to_closest {
                    max_distance_to_closest = dist2_max;
                }

                Some(MinNode {
                    dist2: dist2_min,
                    expanded,
                    value,
                })
            }
            Expanded::Leaf(leaf) => {
                #[allow(clippy::cast_sign_loss)]
                let dist2 = leaf.point.as_ivec2().distance_squared(input.as_ivec2()) as u32;

                if max_distance_to_closest < dist2 {
                    return None;
                }

                if dist2 < max_distance_to_closest {
                    max_distance_to_closest = dist2;
                }

                Some(MinNode {
                    dist2,
                    expanded,
                    value,
                })
            }
        };

        if self.data.is_empty() {
            return None;
        }

        let mut heap: heapless::BinaryHeap<MinNode, Min, HEAP_SIZE> = heapless::BinaryHeap::new();

        let root = self.root_context();
        let node = unsafe { self.get_node(root.idx as usize) };
        let dist2 = u32::MAX;

        heap.push(MinNode {
            dist2,
            expanded: node.into_expanded(),
            value: root,
        })
        .unwrap();

        while let Some(context) = heap.pop() {
            match context.expanded {
                Expanded::Leaf(leaf) => {
                    let start = leaf.start;
                    let end = self.get_next_data_for_idx(context.value.idx);

                    return Some(start as usize..end);
                }
                Expanded::Aabb(..) => {
                    let left = context.value.left();
                    let node = unsafe { self.get_node(left.idx as usize) };
                    let node = node.into_expanded();
                    if let Some(node) = new_node(left, node) {
                        heap.push(node).unwrap();
                    }

                    let right = context.value.right();
                    let node = unsafe { self.get_node(right.idx as usize) };
                    let node = node.into_expanded();
                    if let Some(node) = new_node(right, node) {
                        heap.push(node).unwrap();
                    }
                }
            }
        }

        None
    }

    pub fn get_in(&self, query: Aabb) -> ArrayVec<Range<usize>, DFS_STACK_SIZE> {
        let mut to_send_indices: ArrayVec<Range<usize>, MAX_SIZE> = ArrayVec::new();

        if self.data.is_empty() {
            // nothing
            return to_send_indices;
        }

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
