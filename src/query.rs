use std::alloc::Allocator;
use std::ops::Range;

use arrayvec::ArrayVec;
use glam::I16Vec2;
use heapless::binary_heap::Min;

use crate::aabb::Aabb;
use crate::node::Expanded;
use crate::{child_left, child_right, Bvh, ROOT_IDX};

const MAX_SIZE: usize = 32;
const DFS_STACK_SIZE: usize = 32;
const HEAP_SIZE: usize = 32;

impl<T, A: Allocator> Bvh<T, A> {
    pub fn get_closest_slice(&self, input: I16Vec2) -> Option<&[T]> {
        let idx = self.get_closest(input)?;
        let idx = idx.start as usize..idx.end as usize;
        Some(&self.data[idx])
    }

    /// # Panics
    /// If there are too many elements that overflow `HEAP_SIZE`
    pub fn get_closest(&self, input: I16Vec2) -> Option<Range<u32>> {
        #[derive(Debug, Copy, Clone)]
        struct MinNode {
            dist2: u32,
            expanded: Expanded,
            idx: u32,
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

        let mut new_node = |idx: u32, expanded: Expanded| match expanded {
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
                    idx,
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
                    idx,
                })
            }
        };

        if self.data.is_empty() {
            return None;
        }

        let mut heap: heapless::BinaryHeap<MinNode, Min, HEAP_SIZE> = heapless::BinaryHeap::new();

        // root idx
        let node = unsafe { self.get_node(ROOT_IDX) };
        let dist2 = u32::MAX;

        heap.push(MinNode {
            dist2,
            expanded: node.into_expanded(),
            idx: ROOT_IDX,
        })
        .unwrap();

        while let Some(context) = heap.pop() {
            match context.expanded {
                Expanded::Leaf(leaf) => {
                    let ptr = leaf.ptr;
                    let start = self.leafs[ptr as usize].element_index;
                    let end = self.leafs[ptr as usize + 1].element_index;

                    return Some(start..end);
                }
                Expanded::Aabb(..) => {
                    let left = child_left(context.idx);
                    let node = unsafe { self.get_node(left) };
                    let node = node.into_expanded();
                    if let Some(node) = new_node(left, node) {
                        heap.push(node).unwrap();
                    }

                    let right = child_right(context.idx);
                    let node = unsafe { self.get_node(right) };
                    let node = node.into_expanded();
                    if let Some(node) = new_node(right, node) {
                        heap.push(node).unwrap();
                    }
                }
            }
        }

        None
    }

    pub fn get_in_slices(&self, query: Aabb) -> ArrayVec<&[T], DFS_STACK_SIZE> {
        self.get_in(query)
            .into_iter()
            .map(|range| &self.data[range.start as usize..range.end as usize])
            .collect()
    }

    pub fn get_in(&self, query: Aabb) -> ArrayVec<Range<u32>, DFS_STACK_SIZE> {
        let mut to_send_indices: ArrayVec<Range<u32>, MAX_SIZE> = ArrayVec::new();

        if self.data.is_empty() {
            // nothing
            return to_send_indices;
        }

        let mut dfs_stack: ArrayVec<u32, DFS_STACK_SIZE> = ArrayVec::new();

        // so we do not need special case (there is always a last)
        to_send_indices.push(0..0);
        dfs_stack.push(ROOT_IDX);

        while let Some(idx) = dfs_stack.pop() {
            let node = unsafe { self.get_node(idx) };

            match node.into_expanded() {
                Expanded::Leaf(leaf) => {
                    if !query.contains_point(leaf.point) {
                        continue;
                    }

                    let ptr = leaf.ptr;

                    let start = self.leafs[ptr as usize].element_index;
                    let end = self.leafs[ptr as usize + 1].element_index;

                    let last = unsafe { to_send_indices.last_mut().unwrap_unchecked() };

                    if last.end == start {
                        // combine
                        last.end = end;
                    } else {
                        to_send_indices.push(start..end);
                    }
                }
                Expanded::Aabb(aabb) => {
                    if !aabb.intersects(query) {
                        continue;
                    }

                    let left = child_left(idx);
                    let right = child_right(idx);

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
