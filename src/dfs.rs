pub mod context;

#[allow(clippy::cast_possible_truncation)]
pub const fn depth_for_leaf_node_count(leaf_node_count: u32) -> u8 {
    leaf_node_count.trailing_zeros() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_node_count_to_depth() {
        assert_eq!(depth_for_leaf_node_count(1), 0);
        /*
         * Tree (depth 0):
         *   0
         */

        assert_eq!(depth_for_leaf_node_count(2), 1);
        /*
         * Tree (depth 1):
         *     0
         *    / \
         *   1   2
         */

        assert_eq!(depth_for_leaf_node_count(4), 2);
        /*
         * Tree (depth 2):
         *        0
         *      /   \
         *     1     4
         *    / \   / \
         *   2   3 5   6
         */

        assert_eq!(depth_for_leaf_node_count(8), 3);
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
    }
}
