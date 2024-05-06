use bvh::{Aabb, Bvh, Data, Point};
use glam::I16Vec2;

type EntityId = u32;

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
Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(3, 3) })
  Internal(Aabb { min: I16Vec2(2, 2), max: I16Vec2(3, 3) })
    Leaf([3, 3] -> 3)
    Leaf([2, 2] -> 2)
  Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(1, 1) })
    Leaf([1, 1] -> 1)
    Leaf([0, 0] -> 0)
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
Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(4, 4) })
  Internal(Aabb { min: I16Vec2(2, 2), max: I16Vec2(4, 4) })
    Internal(Aabb { min: I16Vec2(3, 3), max: I16Vec2(4, 4) })
      Leaf([4, 4] -> 4)
      Leaf([3, 3] -> 3)
    Leaf([2, 2] -> 2)
  Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(1, 1) })
    Leaf([1, 1] -> 1)
    Leaf([0, 0] -> 0)
    "
    .trim();

    assert_eq!(s, expected);
}

#[test]
fn test_fuzz() {
    fastrand::seed(3);

    for _ in 0..10 {
        let num_elems = fastrand::usize(..100);
        
        
        let elems: Vec<_> = (0..num_elems)
            .map(|id| Player {
                location: I16Vec2::new(fastrand::i16(-200..200), fastrand::i16(-200..200)),
                id: id as u32,
            })
            .collect();

        println!("built with {} elems", elems.len());
        
        

        let size_hint = elems.len();

        let bvh = Bvh::<u32>::build(elems, size_hint);

        // printout
        let printed = bvh.print();
        println!("{printed}");

        for _ in 0..1000 {
            let point = I16Vec2::new(fastrand::i16(-200..200), fastrand::i16(-200..200));
            println!("querying for {:?}", point);

            let result: Vec<_> = bvh.get_closest_slice(point).unwrap().into_iter().collect();
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
Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(5, 5) })
  Internal(Aabb { min: I16Vec2(3, 3), max: I16Vec2(5, 5) })
    Internal(Aabb { min: I16Vec2(4, 4), max: I16Vec2(5, 5) })
      Leaf([5, 5] -> 5)
      Leaf([4, 4] -> 4)
    Leaf([3, 3] -> 3)
  Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(2, 2) })
    Internal(Aabb { min: I16Vec2(1, 1), max: I16Vec2(2, 2) })
      Leaf([2, 2] -> 2)
      Leaf([1, 1] -> 1)
    Leaf([0, 0] -> 0)
    "
    .trim();

    assert_eq!(s, expected);
}
