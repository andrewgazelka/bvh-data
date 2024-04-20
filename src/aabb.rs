//! A Minecraft world is 60 million blocks long.
//! Each chunk is 16 blocks long.
//! This means that the chunk length is 60 million / 16 = 3,750,000 chunks
//! 2^16 = 65,536 and 2^32 is 4,294,967,296.
//!
//! While we should be using more than i16 for chunk coordinates, this is for minigame servers and we are fine
//! using it as we are optimizing for performance
use crate::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aabb {
    // 64 bit
    min: glam::I16Vec2, // 32 bit
    max: glam::I16Vec2, // 32 bit
}

impl Aabb {
    pub const fn new(min: glam::I16Vec2, max: glam::I16Vec2) -> Self {
        Self { min, max }
    }

    pub fn is_unit(self) -> bool {
        self.min == self.max
    }

    pub fn enclosing_aabb<I: Point>(elems: &[I]) -> Self {
        // 16 bits
        let mut min = glam::I16Vec2::MAX;
        let mut max = glam::I16Vec2::MIN;

        for elem in elems {
            let elem = elem.point();
            min = min.min(elem);
            max = max.max(elem);
        }

        Self::new(min, max)
    }

    pub const fn lens(self) -> [u16; 2] {
        let lx = self.max.x.abs_diff(self.min.x);
        let ly = self.max.y.abs_diff(self.min.y);
        [lx, ly]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lens() {
        let aabb = Aabb::new(glam::I16Vec2::new(0, 0), glam::I16Vec2::new(10, 10));
        let lens = aabb.lens();
        assert_eq!(lens[0], 10);
        assert_eq!(lens[1], 10);
    }

    #[test]
    fn test_enclosing_aabb() {
        let points = vec![
            glam::I16Vec2::new(1, 2),
            glam::I16Vec2::new(3, 4),
            glam::I16Vec2::new(5, 6),
        ];

        let aabb = Aabb::enclosing_aabb(&points);
        assert_eq!(aabb.min, glam::I16Vec2::new(1, 2));
        assert_eq!(aabb.max, glam::I16Vec2::new(5, 6));
    }

    #[test]
    fn test_lens_negative() {
        let aabb = Aabb::new(glam::I16Vec2::new(-10, -10), glam::I16Vec2::new(10, 10));
        let lens = aabb.lens();
        assert_eq!(lens[0], 20);
        assert_eq!(lens[1], 20);
    }

    #[test]
    fn test_lens_zero() {
        let aabb = Aabb::new(glam::I16Vec2::new(0, 0), glam::I16Vec2::new(0, 0));
        let lens = aabb.lens();
        assert_eq!(lens[0], 0);
        assert_eq!(lens[1], 0);
    }
}
