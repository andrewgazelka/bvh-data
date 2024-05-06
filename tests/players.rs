use bvh::{Aabb, Bvh, Data, Point};
use glam::I16Vec2;
use itertools::Itertools;
use more_asserts::assert_le;
use std::collections::HashMap;

type EntityId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Player {
    location: I16Vec2,
    id: EntityId,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            location: I16Vec2::new(0, 0),
            id: 0,
        }
    }
}

impl Point for Player {
    // todo: test returning val vs ref
    fn point(&self) -> I16Vec2 {
        self.location
    }
}

impl Data for Player {
    type Unit = EntityId;

    fn data(&self) -> &[EntityId] {
        core::slice::from_ref(&self.id)
    }
}

#[test]
fn test_local_player() {
    let id = 123;
    let player = Player {
        location: I16Vec2::new(1, 2),
        id,
    };

    assert_eq!(player.point(), I16Vec2::new(1, 2));
    assert_eq!(player.data(), &[id]);
}

#[test]
fn test_build_bvh_with_local_player() {
    let input = vec![
        Player {
            location: I16Vec2::new(0, 0),
            id: 1,
        },
        Player {
            location: I16Vec2::new(1, 1),
            id: 2,
        },
        Player {
            location: I16Vec2::new(2, 2),
            id: 3,
        },
        Player {
            location: I16Vec2::new(3, 3),
            id: 4,
        },
    ];

    let size_hint = input.len();

    let bvh = Bvh::<EntityId>::build(input, size_hint);

    // Check the number of elements in the BVH
    assert_eq!(bvh.elements().len(), 4);

    // Check the contents of the elements
    assert_eq!(bvh.elements(), [1, 2, 3, 4]);

    // Print it out
    let s = bvh.print();

    let expected = r"
01	Internal([0, 0] -> [3, 3])
03	  Internal([2, 2] -> [3, 3])
07	    Leaf([3, 3] -> 3)
06	    Leaf([2, 2] -> 2)
02	  Internal([0, 0] -> [1, 1])
05	    Leaf([1, 1] -> 1)
04	    Leaf([0, 0] -> 0)
    "
    .trim();

    assert_eq!(s, expected);
}

#[test]
fn test_query_single_player() {
    let player = Player {
        location: I16Vec2::new(1, 2),
        id: 123,
    };

    let input = vec![player];
    let size_hint = input.len();

    let bvh = Bvh::<EntityId>::build(input, size_hint);

    // Query the exact location of the player
    let query = Aabb::new(I16Vec2::new(1, 2), I16Vec2::new(1, 2));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..1]);

    // Query a location that doesn't intersect with the player
    let query = Aabb::new(I16Vec2::new(10, 10), I16Vec2::new(10, 10));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..0]);
}

#[test]
fn test_query_multiple_players() {
    //              1
    //          2       3
    //        4   5    6   7
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //

    let input = vec![
        Player {
            location: I16Vec2::new(0, 0),
            id: 1,
        },
        Player {
            location: I16Vec2::new(1, 1),
            id: 2,
        },
        Player {
            location: I16Vec2::new(2, 2),
            id: 3,
        },
        Player {
            location: I16Vec2::new(3, 3),
            id: 4,
        },
    ];

    let size_hint = input.len();

    let bvh = Bvh::<EntityId>::build(input, size_hint);

    // Query a location that intersects with multiple players
    let query = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(2, 2));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..3]);

    // Query a location that intersects with a single player
    let query = Aabb::new(I16Vec2::new(3, 3), I16Vec2::new(3, 3));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..0, 3..4]);

    // Query a location that doesn't intersect with any players
    let query = Aabb::new(I16Vec2::new(10, 10), I16Vec2::new(10, 10));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..0]);
}

#[test]
fn test_build_bvh_with_odd_number_of_players() {
    let input = vec![
        Player {
            location: I16Vec2::new(0, 0),
            id: 1,
        },
        Player {
            location: I16Vec2::new(1, 1),
            id: 2,
        },
        Player {
            location: I16Vec2::new(2, 2),
            id: 3,
        },
        Player {
            location: I16Vec2::new(3, 3),
            id: 4,
        },
        Player {
            location: I16Vec2::new(4, 4),
            id: 5,
        },
    ];

    let size_hint = input.len();

    let bvh = Bvh::<EntityId>::build(input, size_hint);

    // Check the number of elements in the BVH
    assert_eq!(bvh.elements().len(), 5);

    // Check the contents of the elements
    assert_eq!(bvh.elements(), [1, 2, 3, 4, 5]);

    // Print it out
    let s = bvh.print();

    let expected = r"
01	Internal([0, 0] -> [4, 4])
03	  Leaf([4, 4] -> 4)
02	  Internal([0, 0] -> [3, 3])
05	    Internal([2, 2] -> [3, 3])
11	      Leaf([3, 3] -> 3)
10	      Leaf([2, 2] -> 2)
04	    Internal([0, 0] -> [1, 1])
09	      Leaf([1, 1] -> 1)
08	      Leaf([0, 0] -> 0)
    "
    .trim();

    assert_eq!(s, expected);
}

#[test]
fn test_fuzz() {
    fastrand::seed(3);

    for _ in 0..10 {
        let num_elems = fastrand::u32(..100);

        let elems: Vec<_> = (0..num_elems)
            .map(|id| Player {
                location: I16Vec2::new(fastrand::i16(-200..200), fastrand::i16(-200..200)),
                id,
            })
            .collect();

        let size_hint = elems.len();

        let bvh = Bvh::<u32>::build(elems.clone(), size_hint);

        assert_eq!(bvh.elements().len(), elems.len());

        let input_elements_count: HashMap<_, _> = elems
            .iter()
            .map(|x| x.id)
            .group_by(|&x| x)
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect();

        let bvh_elements_count: HashMap<_, _> = bvh
            .elements()
            .iter()
            .group_by(|&x| x)
            .into_iter()
            .map(|(key, group)| (*key, group.count()))
            .collect();

        assert_eq!(input_elements_count, bvh_elements_count);

        for _ in 0..1000 {
            let point = I16Vec2::new(fastrand::i16(-200..200), fastrand::i16(-200..200));

            // todo: check
            let result = bvh.get_closest(point).unwrap();

            let len = result.len();
            assert_le!(result.start, result.end);
            assert_le!(result.start, num_elems);
            assert_le!(len, 3); // it wouldn't make sense if we have more than 3 duplicate points
            assert_le!(result.end, num_elems);
        }
    }
}

#[test]
fn test_closest_player() {
    let input = vec![
        Player {
            location: I16Vec2::new(0, 0),
            id: 1,
        },
        Player {
            location: I16Vec2::new(1, 1),
            id: 2,
        },
        Player {
            location: I16Vec2::new(2, 2),
            id: 3,
        },
        Player {
            location: I16Vec2::new(3, 3),
            id: 4,
        },
        Player {
            location: I16Vec2::new(4, 4),
            id: 5,
        },
    ];

    let size_hint = input.len();

    let bvh = Bvh::<EntityId>::build(input, size_hint);

    //     assert_eq!(
    //         bvh.print(),
    //         r"
    // 01	Internal([0, 0] -> [4, 4])
    // 03	  Internal([2, 2] -> [4, 4])
    // 07	    Internal([3, 3] -> [4, 4])
    // 15	      Leaf([4, 4] -> 4)
    // 14	      Leaf([3, 3] -> 3)
    // 06	    Leaf([2, 2] -> 2)
    // 02	  Internal([0, 0] -> [1, 1])
    // 05	    Leaf([1, 1] -> 1)
    // 04	    Leaf([0, 0] -> 0)
    //     "
    //         .trim()
    //     );

    let result = bvh.get_closest_slice(I16Vec2::new(2, 2)).unwrap();
    assert_eq!(result, &[3]); // id 2
}

#[test]
fn test_build_bvh_with_non_power_of_2_players() {
    let input = vec![
        Player {
            location: I16Vec2::new(0, 0),
            id: 1,
        },
        Player {
            location: I16Vec2::new(1, 1),
            id: 2,
        },
        Player {
            location: I16Vec2::new(2, 2),
            id: 3,
        },
        Player {
            location: I16Vec2::new(3, 3),
            id: 4,
        },
        Player {
            location: I16Vec2::new(4, 4),
            id: 5,
        },
        Player {
            location: I16Vec2::new(5, 5),
            id: 6,
        },
    ];

    let size_hint = input.len();

    let bvh = Bvh::<EntityId>::build(input, size_hint);

    // Check the number of elements in the BVH
    assert_eq!(bvh.elements().len(), 6);

    // Check the contents of the elements
    assert_eq!(bvh.elements(), [1, 2, 3, 4, 5, 6]);

    // Print it out
    let s = bvh.print();

    let expected = r"
01	Internal([0, 0] -> [5, 5])
03	  Internal([4, 4] -> [5, 5])
07	    Leaf([5, 5] -> 5)
06	    Leaf([4, 4] -> 4)
02	  Internal([0, 0] -> [3, 3])
05	    Internal([2, 2] -> [3, 3])
11	      Leaf([3, 3] -> 3)
10	      Leaf([2, 2] -> 2)
04	    Internal([0, 0] -> [1, 1])
09	      Leaf([1, 1] -> 1)
08	      Leaf([0, 0] -> 0)
    "
    .trim();

    assert_eq!(s, expected);
}
