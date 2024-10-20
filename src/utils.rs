use crate::aabb::Aabb;
pub use crate::utils::array::NonZeroArrayExt;
use crate::Point;

mod array;

pub fn partition_index_by_largest_axis<T: Point>(
    elements: &mut [T],
    aabb: Aabb,
) -> (&mut [T], &mut [T]) {
    let lens = aabb.lens();
    let select_idx = lens.max_index();

    let median_idx = elements.len().next_power_of_two() / 2;

    elements.select_nth_unstable_by(median_idx, |a, b| {
        // todo: bench efficiency of not having checked Index
        let a = a.point()[select_idx];
        let b = b.point()[select_idx];

        unsafe { a.partial_cmp(&b).unwrap_unchecked() }
    });

    elements.split_at_mut(median_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::I16Vec2;

    struct TestPoint(I16Vec2);

    impl Point for TestPoint {
        fn point(&self) -> I16Vec2 {
            self.0
        }
    }

    #[test]
    fn test_sort_by_largest_axis_x() {
        let mut elements = vec![
            TestPoint(I16Vec2::new(3, 1)),
            TestPoint(I16Vec2::new(1, 2)),
            TestPoint(I16Vec2::new(4, 3)),
            TestPoint(I16Vec2::new(2, 4)),
        ];
        let aabb = Aabb::new(I16Vec2::new(0, 3), I16Vec2::new(5, 5));
        partition_index_by_largest_axis(&mut elements, aabb);
        assert_eq!(elements[0].point(), I16Vec2::new(1, 2));
        assert_eq!(elements[1].point(), I16Vec2::new(2, 4));
        assert_eq!(elements[2].point(), I16Vec2::new(3, 1));
        assert_eq!(elements[3].point(), I16Vec2::new(4, 3));
    }

    #[test]
    fn test_sort_by_largest_axis_y() {
        let mut elements = vec![
            TestPoint(I16Vec2::new(1, 3)),
            TestPoint(I16Vec2::new(2, 1)),
            TestPoint(I16Vec2::new(3, 4)),
            TestPoint(I16Vec2::new(4, 2)),
        ];
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(5, 6));
        partition_index_by_largest_axis(&mut elements, aabb);
        assert_eq!(elements[0].point(), I16Vec2::new(2, 1));
        assert_eq!(elements[1].point(), I16Vec2::new(4, 2));
        assert_eq!(elements[2].point(), I16Vec2::new(1, 3));
        assert_eq!(elements[3].point(), I16Vec2::new(3, 4));
    }

    #[test]
    fn test_sort_by_largest_axis_equal_dimensions() {
        let mut elements = vec![
            TestPoint(I16Vec2::new(1, 1)),
            TestPoint(I16Vec2::new(2, 2)),
            TestPoint(I16Vec2::new(3, 3)),
            TestPoint(I16Vec2::new(4, 4)),
        ];
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(5, 5));
        partition_index_by_largest_axis(&mut elements, aabb);
        assert_eq!(elements[0].point(), I16Vec2::new(1, 1));
        assert_eq!(elements[1].point(), I16Vec2::new(2, 2));
        assert_eq!(elements[2].point(), I16Vec2::new(3, 3));
        assert_eq!(elements[3].point(), I16Vec2::new(4, 4));
    }

    // expect error
    #[test]
    #[should_panic(expected = "partition_at_index index 0 greater than length of slice 0")]
    fn test_sort_by_largest_axis_empty() {
        let mut elements: Vec<TestPoint> = vec![];
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(5, 5));
        partition_index_by_largest_axis(&mut elements, aabb);
    }

    #[test]
    fn test_sort_by_largest_axis_single_element() {
        let mut elements = vec![TestPoint(I16Vec2::new(3, 4))];
        let aabb = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(5, 5));
        partition_index_by_largest_axis(&mut elements, aabb);
        assert_eq!(elements[0].point(), I16Vec2::new(3, 4));
    }
}
