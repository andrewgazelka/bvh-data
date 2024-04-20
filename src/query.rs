use crate::aabb::Aabb;
use crate::node::NodeExpanded;
use crate::Bvh;
use std::alloc::Allocator;
use std::ops::Range;

// impl<T, A: Allocator> Bvh<T, A> {
//     fn get_in(&self, aaab: Aabb) -> Range<usize> {
//         let root_context = self.root_context();
//
//         // todo: handle no root
//         let root = unsafe { self.get_node(root_context.idx as usize) };
//
//         match root.into_expanded() {
//             NodeExpanded::Aabb(bounding) => {
//                 if bounding.collides(aaab) {
//                     // go down
//                 }
//             }
//             NodeExpanded::Leaf(leaf) => {
//
//             }
//         }
//
//         todo!()
//     }
// }
