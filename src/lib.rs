#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(allocator_api)]
#![feature(new_uninit)]

use std::alloc::{Allocator, Global};
use std::cell::Cell;
use std::num::NonZeroU32;

pub use crate::aabb::Aabb;
use crate::node::{Expanded, Node};
use crate::sealed::PointWithData;
use crate::utils::partition_index_by_largest_axis;

mod aabb;

mod node;
mod print;
mod query;
mod utils;

//         1
//      2     3
//          4   5
//

pub struct Bvh<T, A: Allocator = Global> {
    nodes: Box<[Cell<Node>], A>,
    data: Vec<T, A>,
}

impl<T, A: Allocator + Default> Default for Bvh<T, A> {
    fn default() -> Self {
        Self {
            // zeroed so everything is equivalent to Aabb with 0,0
            nodes: Box::new_in([], A::default()),
            data: Vec::with_capacity_in(0, A::default()),
        }
    }
}

// broadcast buffer &[1,2,3,4,5,6]
// packet info

pub trait Point {
    /// Generally, this will be an [`u8`]
    fn point(&self) -> glam::I16Vec2;
}

impl Point for glam::I16Vec2 {
    fn point(&self) -> glam::I16Vec2 {
        *self
    }
}

pub trait Data {
    type Unit;
    fn data(&self) -> &[Self::Unit];
}

mod sealed {
    use crate::{Data, Point};

    pub trait PointWithData: Point + Data {}
}

impl<T> sealed::PointWithData for T where T: Point + Data {}

impl<T> Bvh<T> {
    #[must_use]
    pub fn build<I>(input: Vec<I>, size_hint: usize) -> Self
    where
        I: PointWithData<Unit = T>,
        T: Copy + 'static,
    {
        Self::build_in(input, size_hint, Global)
    }
}

const fn round_power_of_two(mut x: usize) -> usize {
    if !x.is_power_of_two() {
        x = x.next_power_of_two();
    }
    x
}

impl<T, A: Allocator + Clone> Bvh<T, A> {
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn build_in<I>(mut input: Vec<I>, size_hint: usize, alloc: A) -> Self
    where
        I: PointWithData<Unit = T>,
        T: Copy + 'static,
    {
        if input.is_empty() {
            return Self {
                nodes: Box::new_in([], alloc.clone()),
                data: Vec::new_in(alloc),
            };
        }

        // we will have max input.len() leaf nodes
        let leaf_node_count = round_power_of_two(input.len());
        let total_nodes_len = leaf_node_count * 2 - 1;

        let mut bvh = Self {
            nodes: unsafe {
                Box::new_zeroed_slice_in(total_nodes_len, alloc.clone()).assume_init()
            },
            data: Vec::with_capacity_in(size_hint, alloc),
        };

        build_bvh_helper(&mut bvh, &mut input, ROOT_IDX);

        bvh
    }
}

impl<T, A: Allocator> Bvh<T, A> {
    fn set_node(&self, idx: u32, node: Node) {
        // todo: I think this is safe write, right?
        let ptr = &self.nodes[idx as usize - 1];
        ptr.set(node);
    }

    pub fn elements(&self) -> &[T] {
        &self.data
    }

    unsafe fn get_node(&self, idx: u32) -> Node {
        let ptr = &self.nodes[idx as usize - 1];
        ptr.get()
    }

    unsafe fn get_node_opt(&self, idx: u32) -> Option<Node> {
        let ptr = self.nodes.get(idx as usize - 1)?;
        Some(ptr.get())
    }

    // todo: this is impl pretty inefficiently. I feel there is an O(1) approach but I cannot think of it right now
    pub fn get_next_data_for_idx(&self, idx: u32) -> u32 {
        // todo: is there a more efficient way to do this?
        #[allow(clippy::cast_possible_truncation)]
        let Some(sibling_right) = sibling_right(idx) else {
            return self.elements().len() as u32;
        };

        let start = sibling_right.get();

        let mut on = start;

        // try to look down
        while let Some(node) = unsafe { self.get_node_opt(on) } {
            if let Expanded::Leaf(leaf) = node.into_expanded() {
                return leaf.start;
            }
            on = child_left(on);
        }

        on = start;
        // try to look up
        loop {
            let Some(parent) = parent(on) else {
                unreachable!("This should never occur")
            };

            let parent = parent.get();

            let node = unsafe { self.get_node(parent) };
            let node = node.into_expanded();
            if let Expanded::Leaf(leaf) = node {
                return leaf.start;
            }
            on = parent;
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn build_bvh_helper<T: PointWithData, A: Allocator>(
    build: &mut Bvh<T::Unit, A>,
    elements: &mut [T],
    current_idx: u32,
) where
    T::Unit: Copy + 'static,
{
    let len = elements.len();

    debug_assert!(len != 0, "trying to build a BVH with no nodes");

    let aabb = Aabb::enclosing_aabb(elements);

    if let Some(point) = aabb.to_unit() {
        // this is a leaf node
        let start_index = build.data.len();

        for elem in elements {
            build.data.extend_from_slice(elem.data());
        }

        let node = Node::leaf(point, start_index as u32);
        build.set_node(current_idx, node);

        return;
    }

    build.set_node(current_idx, Node::aabb(aabb));

    let left_context = child_left(current_idx);
    let right_context = child_right(current_idx);

    let (left, right) = partition_index_by_largest_axis(elements, aabb);

    build_bvh_helper(build, left, left_context);
    build_bvh_helper(build, right, right_context);
}

#[must_use]
pub const fn child_left(idx: u32) -> u32 {
    idx * 2
}

#[must_use]
pub const fn parent(idx: u32) -> Option<NonZeroU32> {
    NonZeroU32::new(idx / 2)
}

#[must_use]
pub const fn child_right(idx: u32) -> u32 {
    idx * 2 + 1
}

#[must_use]
pub const fn sibling_right(idx: u32) -> Option<NonZeroU32> {
    //      1
    //    2   3
    //   45   67

    let tentative_next = idx + 1;

    if tentative_next.is_power_of_two() {
        // there is nothing to the right, we would be going to the next row
        None
    } else {
        Some(unsafe { NonZeroU32::new_unchecked(tentative_next) })
    }
}

const ROOT_IDX: u32 = 1;

#[cfg(test)]
mod tests {
    use crate::sibling_right;
    use std::num::NonZeroU32;

    #[test]
    fn test_sibling_right() {
        assert_eq!(sibling_right(1), None);

        assert_eq!(sibling_right(2), NonZeroU32::new(3));
        assert_eq!(sibling_right(3), None);

        assert_eq!(sibling_right(4), NonZeroU32::new(5));
        assert_eq!(sibling_right(5), NonZeroU32::new(6));
        assert_eq!(sibling_right(6), NonZeroU32::new(7));
        assert_eq!(sibling_right(7), None);
    }
}
