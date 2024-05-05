#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(allocator_api)]

pub use crate::aabb::Aabb;
use crate::dfs::context::Dfs;
use crate::dfs::depth_for_leaf_node_count;
use crate::node::{Expanded, Node};
use std::alloc::{Allocator, Global};
use std::cell::Cell;
use std::mem::MaybeUninit;

use crate::sealed::PointWithData;
use crate::utils::partition_index_by_largest_axis;

mod aabb;

mod dfs;
mod node;
mod print;
mod query;
mod utils;

pub struct Bvh<T, A: Allocator = Global> {
    nodes: Box<[MaybeUninit<Cell<Node>>], A>,
    data: Vec<T, A>,
    depth: u8,
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
        // we will have max input.len() leaf nodes
        let leaf_node_count = round_power_of_two(input.len());
        let total_nodes_len = leaf_node_count * 2 - 1;
        let depth = depth_for_leaf_node_count(leaf_node_count as u32);

        let context = Dfs::new(depth);

        let mut bvh = Self {
            nodes: Box::new_uninit_slice_in(total_nodes_len, alloc.clone()),
            data: Vec::with_capacity_in(size_hint, alloc),
            depth,
        };

        build_bvh_helper(&mut bvh, &mut input, context);

        bvh
    }
}

impl<T, A: Allocator> Bvh<T, A> {
    fn set_node(&self, idx: usize, node: Node) {
        // todo: I think this is safe write, right?
        let ptr = self.nodes[idx].as_ptr();
        let ptr = unsafe { &*ptr };
        ptr.set(node);
    }

    const fn root_context(&self) -> Dfs {
        Dfs::new(self.depth)
    }

    pub fn elements(&self) -> &[T] {
        &self.data
    }

    unsafe fn get_node(&self, idx: usize) -> Node {
        let ptr = self.nodes[idx].as_ptr();
        let ptr = &*ptr;
        ptr.get()
    }

    unsafe fn get_opt_node(&self, idx: usize) -> Option<Node> {
        let ptr = self.nodes.get(idx)?.as_ptr();
        let ptr = &*ptr;
        Some(ptr.get())
    }

    // todo: this is impl pretty inefficiently. I feel there is an O(1) approach but I cannot think of it right now
    pub fn get_next_data_for_idx(&self, idx: u32) -> usize {
        let mut idx_on = (idx + 1) as usize;

        loop {
            let node = unsafe { self.get_opt_node(idx_on) };

            let Some(node) = node else {
                return self.data.len();
            };

            let expanded = node.into_expanded();

            if let Expanded::Leaf(leaf) = expanded {
                return leaf.start as usize;
            }

            idx_on += 1;
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn build_bvh_helper<T: PointWithData, A: Allocator>(
    build: &mut Bvh<T::Unit, A>,
    elements: &mut [T],
    context: Dfs,
) where
    T::Unit: Copy + 'static,
{
    let len = elements.len();

    debug_assert!(len != 0, "trying to build a BVH with no nodes");

    let aabb = Aabb::enclosing_aabb(elements);

    if let Some(point) = aabb.to_unit() {
        let start_index = build.data.len();

        for elem in elements {
            build.data.extend_from_slice(elem.data());
        }

        let node = Node::leaf(point, start_index as u32);
        build.set_node(context.idx as usize, node);

        return;
    }

    build.set_node(context.idx as usize, Node::aabb(aabb));

    let left_context = context.left();
    let right_context = context.right();

    let (left, right) = partition_index_by_largest_axis(elements, aabb);

    build_bvh_helper(build, left, left_context);
    build_bvh_helper(build, right, right_context);
}
