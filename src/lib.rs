#![feature(generic_const_exprs)]
#![feature(allocator_api)]

use crate::aabb::Aabb;
use crate::dfs::context::DfsContext;
use crate::dfs::depth_for_leaf_node_count;
use crate::node::Node;
use std::alloc::{Allocator, Global};
use std::cell::Cell;
use std::mem::MaybeUninit;

use crate::sealed::PointWithData;
use crate::utils::partition_index_by_largest_axis;

mod aabb;
mod dfs;
mod node;
mod print;
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

pub trait Data<'a> {
    type Unit;
    fn data(&self) -> &'a [Self::Unit];
}

mod sealed {
    use crate::{Data, Point};

    pub trait PointWithData<'a>: Point + Data<'a> {}
}

impl<'a, T> sealed::PointWithData<'a> for T where T: Point + Data<'a> {}

impl<T> Bvh<T> {
    #[must_use]
    pub fn build<'a, I>(input: Vec<I>, size_hint: usize) -> Self
    where
        I: PointWithData<'a, Unit = T>,
        T: Copy + 'static,
    {
        Self::build_in(input, size_hint, Global)
    }
}

impl<T, A: Allocator + Clone> Bvh<T, A> {
    #[must_use]
    pub fn build_in<'a, I>(mut input: Vec<I>, size_hint: usize, alloc: A) -> Self
    where
        I: PointWithData<'a, Unit = T>,
        T: Copy + 'static,
    {
        // we will have max input.len() leaf nodes
        let leaf_node_count = input.len();
        let total_nodes_len = input.len() * 2 - 1;
        let depth = depth_for_leaf_node_count(leaf_node_count as u32);

        let context = DfsContext::new(depth);

        let mut bvh = Self {
            nodes: Box::new_uninit_slice_in(total_nodes_len + 1, alloc.clone()),
            data: Vec::with_capacity_in(size_hint, alloc),
            depth,
        };

        build_bvh_helper(&mut bvh, 0, &mut input, context);

        // so we can handle the edge case where we are getting the last elem
        let elements_len = bvh.data.len();
        bvh.nodes
            .last_mut()
            .unwrap()
            .write(Cell::new(Node::leaf(elements_len as u32)));

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

    const fn root_context(&self) -> DfsContext {
        DfsContext::new(self.depth)
    }

    pub fn elements(&self) -> &[T] {
        &self.data
    }
}

fn build_bvh_helper<'a, T: PointWithData<'a>, A: Allocator>(
    build: &mut Bvh<T::Unit, A>,
    elements_start_idx: u32, // todo: remove could just use a dif repr of a slice
    elements: &mut [T],
    context: DfsContext,
) where
    T::Unit: Copy + 'static,
{
    let len = elements.len();

    debug_assert!(len != 0, "trying to build a BVH with no nodes");
    debug_assert!(
        len.is_power_of_two(),
        "we are using maths that are easier with perfectly filled trees"
    );

    let aabb = Aabb::enclosing_aabb(elements);

    if aabb.is_unit() || len == 1 {
        let insert_elements = &mut build.data;
        for elem in elements {
            insert_elements.extend_from_slice(elem.data());
        }

        let node = Node::leaf(elements_start_idx);

        build.set_node(context.idx as usize, node);

        return;
    }

    let left_context = context.left();
    let right_context = context.right();

    let (left, right) = partition_index_by_largest_axis(elements, aabb);

    build_bvh_helper(build, elements_start_idx, left, left_context);
    build_bvh_helper(
        build,
        elements_start_idx + left.len() as u32,
        right,
        right_context,
    );
}
