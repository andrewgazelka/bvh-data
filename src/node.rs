use crate::aabb::Aabb;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct Two {
    left: u32,
    right: u32,
}

impl Display for Two {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.left, self.right)
    }
}

#[derive(Copy, Clone)]
pub union Node {
    aabb: Aabb,
    two: Two,
}

const _: () = assert!(std::mem::size_of::<Aabb>() == std::mem::size_of::<i64>());
const _: () = assert!(std::mem::size_of::<Node>() == std::mem::size_of::<i64>());

pub struct Leaf {
    pub point: glam::I16Vec2,
    pub start: u32,
}

impl Display for Leaf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.point, self.start)
    }
}

const MSB_1_MASK: u32 = 0x8000_0000;

pub enum NodeExpanded {
    Aabb(Aabb),
    Leaf(Leaf),
}

impl Node {
    // todo: we might be able to only need one index and just look at the next leaf node to determine the
    // range but this might be a bit more complicated.
    // Also it might be needed if we can store two indexes easily.
    pub fn leaf_element_indices(self) -> Option<Leaf> {
        let as_two = unsafe { self.two };

        let msb_left = as_two.left >> 31;
        let msb_right = as_two.right >> 31;

        // if msb_left has 0 and msb_right has 1, then it is a leaf because this means
        // that because of 2's complement, the left is positive and the right is negative
        // meaning that if we are thinking about the Aabb representation, the left min x coord
        // would be greater than the right max x coord which is impossible
        // therefore, we can say that this is a leaf node

        if msb_left == 0 && msb_right == 1 {
            // 0{u31 = left}1{u31 = right}

            // left will look like
            // 0{point_x_16}{start_msb_15}

            // right will look like
            // 0{point_y_16}{start_lsb_15}

            // mask out first bit
            let left = as_two.left & 0x7FFF_FFFF;
            let right = as_two.right & 0x7FFF_FFFF;

            // todo: impl,
            let point_x = (left >> 15) as i16;
            let point_y = (right >> 15) as i16;

            // first 15 bits
            let start = ((left & 0x7FFF) << 15) | (right & 0x7FFF);

            Some(Leaf {
                point: glam::I16Vec2::new(point_x, point_y),
                start,
            })
        } else {
            None
        }
    }

    pub fn into_expanded(self) -> NodeExpanded {
        if let Some(leaf) = self.leaf_element_indices() {
            return NodeExpanded::Leaf(leaf);
        }
        NodeExpanded::Aabb(unsafe { self.aabb })
    }

    pub fn leaf(point: glam::I16Vec2, start: u32) -> Self {
        // make sure start is at most u30::MAX = 2^30 - 1 = 0x3FFFFFFF
        debug_assert!(
            start <= 0x3FFF_FFFF,
            "start must be at most u30::MAX (0x3FFF_FFFF)"
        );

        // if converting directly to u32 will be a different transformation (because of 2's complement)
        let point_x = (point.x as u16) as u32;
        let point_y = (point.y as u16) as u32;

        let left = (point_x << 15) | (start >> 15);
        let right = MSB_1_MASK | (point_y << 15) | (start & 0x7FFF);

        Self {
            two: Two { left, right },
        }
    }

    pub const fn aabb(aabb: Aabb) -> Self {
        // assert that min <= max
        debug_assert!(aabb.min.x <= aabb.max.x);

        // technically this is not needed
        debug_assert!(aabb.min.y <= aabb.max.y);

        Self { aabb }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::I16Vec2;

    #[test]
    fn test_leaf_element_indices_valid() {
        let node = Node::leaf(I16Vec2::new(1, 2), 10);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.point, I16Vec2::new(1, 2));
        assert_eq!(indices.start, 10);
    }

    #[test]
    fn test_leaf_element_indices_invalid() {
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(1, 1));
        let node = Node { aabb };
        assert!(node.leaf_element_indices().is_none());
    }

    #[test]
    fn test_into_expanded_leaf() {
        let node = Node::leaf(I16Vec2::new(1, 2), 10);
        let expanded = node.into_expanded();
        match expanded {
            NodeExpanded::Leaf(leaf) => {
                assert_eq!(leaf.point, I16Vec2::new(1, 2));
                assert_eq!(leaf.start, 10);
            }
            _ => panic!("Expected NodeExpanded::Leaf"),
        }
    }

    // todo: multi test

    #[test]
    fn test_into_expanded_aabb() {
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(1, 1));
        let node = Node { aabb };
        let expanded = node.into_expanded();
        match expanded {
            NodeExpanded::Aabb(aabb) => {
                assert_eq!(aabb.min, I16Vec2::new(0, 0));
                assert_eq!(aabb.max, I16Vec2::new(1, 1));
            }
            _ => panic!("Expected NodeExpanded::Aabb"),
        }
    }

    #[test]
    fn test_leaf_zero_indices() {
        let node = Node::leaf(I16Vec2::new(0, 0), 0);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.point, I16Vec2::new(0, 0));
        assert_eq!(indices.start, 0);
    }

    #[test]
    fn test_leaf_negative_indices() {
        let node = Node::leaf(I16Vec2::new(-1, -2), 0x3FFF_FFFF);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.point, I16Vec2::new(-1, -2));
        assert_eq!(indices.start, 0x3FFF_FFFF);
    }

    #[test]
    fn simple_leaf_fuzz() {
        fastrand::seed(3);

        for _ in 0..1000 {
            let point = I16Vec2::new(fastrand::i16(..), fastrand::i16(..));
            let start = fastrand::u32(..);

            if start > 0x3FFF_FFFF {
                continue;
            }

            let node = Node::leaf(point, start);
            let indices = node.leaf_element_indices().unwrap();
            assert_eq!(indices.point, point);
            assert_eq!(indices.start, start);
        }
    }

    #[test]
    fn test_leaf_max_indices() {
        let node = Node::leaf(I16Vec2::new(i16::MAX, i16::MAX), 0x3FFF_FFFF);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.point, I16Vec2::new(i16::MAX, i16::MAX));
        assert_eq!(indices.start, 0x3FFF_FFFF);
    }

    #[test]
    #[should_panic(expected = "start must be at most u30::MAX (0x3FFF_FFFF)")]
    fn test_leaf_start_overflow() {
        Node::leaf(I16Vec2::new(0, 0), 0x4000_0000);
    }

    #[test]
    #[should_panic(expected = "start must be at most u30::MAX (0x3FFF_FFFF)")]
    fn test_leaf_start_overflow_2() {
        Node::leaf(I16Vec2::new(0, 0), 0x7FFF_FFFF);
    }
}
