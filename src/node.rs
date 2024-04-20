use crate::aabb::Aabb;
use crate::Bvh;

// todo: does this make the most sense to do? could also look at incorrect min, max where they are going the wrong
// way in order to store data
const LEAF_INDICATOR_VALUE: i32 = i32::MIN;

#[derive(Debug, Copy, Clone)]
struct Two {
    indicator: i32,
    right: u32,
}

impl Two {
    const fn leaf_index(index: u32) -> Self {
        Self {
            //
            indicator: LEAF_INDICATOR_VALUE,
            right: index,
        }
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

impl Node {
    pub fn leaf_element_start_idx(self) -> Option<u32> {
        unsafe {
            if self.two.indicator != LEAF_INDICATOR_VALUE {
                return None;
            }
            Some(self.two.right)
        }
    }

    pub fn leaf(idx_start: u32) -> Self {
        Self {
            two: Two::leaf_index(idx_start),
        }
    }

    pub const ZERO: Self = Self { one: 0 };
}
