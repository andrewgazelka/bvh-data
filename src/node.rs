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
    one: u64,
}

const _: () = assert!(std::mem::size_of::<Aabb>() == std::mem::size_of::<i64>());
const _: () = assert!(std::mem::size_of::<Node>() == std::mem::size_of::<i64>());

pub enum NodeExpanded {
    Aabb(Aabb),
    Leaf { start: u32, end: u32 },
}

impl Node {
    // todo: we might be able to only need one index and just look at the next leaf node to determine the
    // range but this might be a bit more complicated.
    // Also it might be needed if we can store two indexes easily.
    pub fn leaf_element_indices(self) -> Option<Two> {
        let as_two = unsafe { self.two };

        let msb_left = as_two.left >> 31;
        let msb_right = as_two.right >> 31;

        // if msb_left has 0 and msb_right has 1, then it is a leaf because this means
        // that because of 2's complement, the left is positive and the right is negative
        // meaning that if we are thinking about the Aabb representation, the left min x coord
        // would be greater than the right max x coord which is impossible
        // therefore, we can say that this is a leaf node

        if msb_left == 0 && msb_right == 1 {
            // mask out first bit
            let left = as_two.left & 0x7FFF_FFFF;
            let right = as_two.right & 0x7FFF_FFFF;

            Some(Two { left, right })
        } else {
            None
        }
    }

    pub fn into_expanded(self) -> NodeExpanded {
        if let Some(two) = self.leaf_element_indices() {
            return NodeExpanded::Leaf {
                start: two.left,
                end: two.right,
            };
        }
        NodeExpanded::Aabb(unsafe { self.aabb })
    }

    pub fn leaf(start: u32, end: u32) -> Self {
        // make sure max u31
        debug_assert!(
            start < 0x8000_0000,
            "start must be at most u31::MAX (0x7FFF_FFFF)"
        );
        debug_assert!(
            end < 0x8000_0000,
            "end must be at most u31::MAX (0x7FFF_FFFF)"
        );

        // we know left already starts with 0 bit
        let left = start;
        let right = end | 0x8000_0000;

        Self {
            two: Two { left, right },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::I16Vec2;

    #[test]
    fn test_leaf_element_indices_valid() {
        let node = Node::leaf(10, 20);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.left, 10);
        assert_eq!(indices.right, 20);
    }

    #[test]
    fn test_leaf_element_indices_invalid() {
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(1, 1));
        let node = Node { aabb };
        assert!(node.leaf_element_indices().is_none());
    }

    #[test]
    fn test_into_expanded_leaf() {
        let node = Node::leaf(10, 20);
        let expanded = node.into_expanded();
        match expanded {
            NodeExpanded::Leaf { start, end } => {
                assert_eq!(start, 10);
                assert_eq!(end, 20);
            }
            _ => panic!("Expected NodeExpanded::Leaf"),
        }
    }

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
        let node = Node::leaf(0, 0);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.left, 0);
        assert_eq!(indices.right, 0);
    }

    #[test]
    fn test_leaf_max_indices() {
        let node = Node::leaf(0x7FFF_FFFF, 0x7FFF_FFFF);
        let indices = node.leaf_element_indices().unwrap();
        assert_eq!(indices.left, 0x7FFF_FFFF);
        assert_eq!(indices.right, 0x7FFF_FFFF);
    }

    #[test]
    #[should_panic(expected = "start must be at most u31::MAX (0x7FFF_FFFF)")]
    fn test_leaf_start_overflow() {
        Node::leaf(0x8000_0000, 0);
    }

    #[test]
    #[should_panic(expected = "end must be at most u31::MAX (0x7FFF_FFFF)")]
    fn test_leaf_end_overflow() {
        Node::leaf(0, 0x8000_0000);
    }
}
