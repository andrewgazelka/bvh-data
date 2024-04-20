#[derive(Copy, Clone)]
pub struct DfsContext {
    pub idx: u32,
    pub distance_to_leaf: u8,
}

impl DfsContext {
    pub const fn new(depth: u8) -> Self {
        Self {
            idx: 0,
            distance_to_leaf: depth,
        }
    }

    pub fn left(self) -> Self {
        debug_assert!(
            self.distance_to_leaf != 0,
            "trying to go left on a leaf node"
        );
        Self {
            idx: self.idx + 1,
            distance_to_leaf: self.distance_to_leaf - 1,
        }
    }

    pub fn right(self) -> Self {
        debug_assert!(
            self.distance_to_leaf != 0,
            "trying to go right on a leaf node"
        );
        Self {
            idx: self.idx + 2_u32.pow(u32::from(self.distance_to_leaf)),
            distance_to_leaf: self.distance_to_leaf - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_0() {
        /*
         * Tree (depth 0):
         *   0
         */
        let context = DfsContext {
            idx: 0,
            distance_to_leaf: 0,
        };

        assert_eq!(context.idx, 0);
        assert_eq!(context.distance_to_leaf, 0);
    }

    #[test]
    fn test_depth_1() {
        /*
         * Tree (depth 1):
         *     0
         *    / \
         *   1   2
         */
        let context = DfsContext {
            idx: 0,
            distance_to_leaf: 1,
        };

        let left = context.left();
        assert_eq!(left.idx, 1);
        assert_eq!(left.distance_to_leaf, 0);

        let right = context.right();
        assert_eq!(right.idx, 2);
        assert_eq!(right.distance_to_leaf, 0);
    }

    #[test]
    fn test_depth_2() {
        /*
         * Tree (depth 2):
         *        0
         *      /   \
         *     1     4
         *    / \   / \
         *   2   3 5   6
         */
        let context = DfsContext {
            idx: 0,
            distance_to_leaf: 2,
        };

        let left = context.left();
        assert_eq!(left.idx, 1);
        assert_eq!(left.distance_to_leaf, 1);

        let left_left = left.left();
        assert_eq!(left_left.idx, 2);
        assert_eq!(left_left.distance_to_leaf, 0);

        let left_right = left.right();
        assert_eq!(left_right.idx, 3);
        assert_eq!(left_right.distance_to_leaf, 0);

        let right = context.right();
        assert_eq!(right.idx, 4);
        assert_eq!(right.distance_to_leaf, 1);

        let right_left = right.left();
        assert_eq!(right_left.idx, 5);
        assert_eq!(right_left.distance_to_leaf, 0);

        let right_right = right.right();
        assert_eq!(right_right.idx, 6);
        assert_eq!(right_right.distance_to_leaf, 0);
    }

    #[test]
    fn test_depth_3() {
        /*
         * Tree (depth 3):
         *            0
         *         /     \
         *        1       8
         *      /  \    /   \
         *     2    5  9    12
         *    / \  / \ / \  / \
         *   3  4 6  7 10 11 13 14
         */
        let context = DfsContext {
            idx: 0,
            distance_to_leaf: 3,
        };

        let left = context.left();
        assert_eq!(left.idx, 1);
        assert_eq!(left.distance_to_leaf, 2);

        let left_left = left.left();
        assert_eq!(left_left.idx, 2);
        assert_eq!(left_left.distance_to_leaf, 1);

        let left_left_left = left_left.left();
        assert_eq!(left_left_left.idx, 3);
        assert_eq!(left_left_left.distance_to_leaf, 0);

        let left_left_right = left_left.right();
        assert_eq!(left_left_right.idx, 4);
        assert_eq!(left_left_right.distance_to_leaf, 0);

        let left_right = left.right();
        assert_eq!(left_right.idx, 5);
        assert_eq!(left_right.distance_to_leaf, 1);

        let left_right_left = left_right.left();
        assert_eq!(left_right_left.idx, 6);
        assert_eq!(left_right_left.distance_to_leaf, 0);

        let left_right_right = left_right.right();
        assert_eq!(left_right_right.idx, 7);
        assert_eq!(left_right_right.distance_to_leaf, 0);

        let right = context.right();
        assert_eq!(right.idx, 8);
        assert_eq!(right.distance_to_leaf, 2);

        let right_left = right.left();
        assert_eq!(right_left.idx, 9);
        assert_eq!(right_left.distance_to_leaf, 1);

        let right_left_left = right_left.left();
        assert_eq!(right_left_left.idx, 10);
        assert_eq!(right_left_left.distance_to_leaf, 0);

        let right_left_right = right_left.right();
        assert_eq!(right_left_right.idx, 11);
        assert_eq!(right_left_right.distance_to_leaf, 0);

        let right_right = right.right();
        assert_eq!(right_right.idx, 12);
        assert_eq!(right_right.distance_to_leaf, 1);

        let right_right_left = right_right.left();
        assert_eq!(right_right_left.idx, 13);
        assert_eq!(right_right_left.distance_to_leaf, 0);

        let right_right_right = right_right.right();
        assert_eq!(right_right_right.idx, 14);
        assert_eq!(right_right_right.distance_to_leaf, 0);
    }
}
