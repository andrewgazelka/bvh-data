use bvh::{Aabb, Bvh, Data, Point};
use glam::I16Vec2;

struct ChunkWithPackets<'a> {
    location: I16Vec2,
    packets_data: &'a [u8],
}

impl<'a> Point for ChunkWithPackets<'a> {
    // todo: test returning val vs ref
    fn point(&self) -> I16Vec2 {
        self.location
    }
}

impl<'a> Data<'a> for ChunkWithPackets<'a> {
    type Unit = u8;

    fn data(&self) -> &'a [u8] {
        self.packets_data
    }
}

#[test]
fn test_local_packet() {
    let data = [1, 2, 3, 4];
    let packet = ChunkWithPackets {
        location: I16Vec2::new(1, 2),
        packets_data: &data,
    };

    assert_eq!(packet.point(), I16Vec2::new(1, 2));
    assert_eq!(packet.data(), &data);
}

#[test]
fn test_build_bvh_with_local_packet() {
    let data1 = [1, 2, 3, 4];
    let data2 = [5, 6, 7, 8];
    let data3 = [9, 10, 11, 12];
    let data4 = [13, 14, 15, 16];

    let input = vec![
        ChunkWithPackets {
            location: I16Vec2::new(0, 0),
            packets_data: &data1,
        },
        ChunkWithPackets {
            location: I16Vec2::new(1, 1),
            packets_data: &data2,
        },
        ChunkWithPackets {
            location: I16Vec2::new(2, 2),
            packets_data: &data3,
        },
        ChunkWithPackets {
            location: I16Vec2::new(3, 3),
            packets_data: &data4,
        },
    ];

    let size_hint = input.len() * 4;

    let bvh = Bvh::<u8>::build(input, size_hint);

    // Check the number of nodes in the BVH
    // assert_eq!(bvh.nodes.len(), 7);

    // Check the number of elements in the BVH
    assert_eq!(bvh.elements().len(), 16);

    // Check the contents of the elements
    assert_eq!(
        bvh.elements(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    );

    // print it out
    let s = bvh.print();

    let expected = r#"
Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(3, 3) })
  Internal(Aabb { min: I16Vec2(0, 0), max: I16Vec2(1, 1) })
  Internal(Aabb { min: I16Vec2(2, 2), max: I16Vec2(3, 3) })
    Leaf([0, 0] -> 0)
    Leaf([1, 1] -> 4)
    Leaf([2, 2] -> 8)
    Leaf([3, 3] -> 12)
    "#
    .trim();

    assert_eq!(s, expected);
}

#[test]
fn test_query_single_packet() {
    let data = [1, 2, 3, 4];
    let packet = ChunkWithPackets {
        location: I16Vec2::new(1, 2),
        packets_data: &data,
    };

    let input = vec![packet];
    let size_hint = input.len() * 4;

    let bvh = Bvh::<u8>::build(input, size_hint);

    // Query the exact location of the packet
    let query = Aabb::new(I16Vec2::new(1, 2), I16Vec2::new(1, 2));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..4]);

    // Query a location that doesn't intersect with the packet
    let query = Aabb::new(I16Vec2::new(10, 10), I16Vec2::new(10, 10));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..0]);
}

#[test]
fn test_query_multiple_packets() {
    let data1 = [1, 2, 3, 4];
    let data2 = [5, 6, 7, 8];
    let data3 = [9, 10, 11, 12];
    let data4 = [13, 14, 15, 16];

    let input = vec![
        ChunkWithPackets {
            location: I16Vec2::new(0, 0),
            packets_data: &data1,
        },
        ChunkWithPackets {
            location: I16Vec2::new(1, 1),
            packets_data: &data2,
        },
        ChunkWithPackets {
            location: I16Vec2::new(2, 2),
            packets_data: &data3,
        },
        ChunkWithPackets {
            location: I16Vec2::new(3, 3),
            packets_data: &data4,
        },
    ];

    let size_hint = input.len() * 4;

    let bvh = Bvh::<u8>::build(input, size_hint);

    // Query a location that intersects with multiple packets
    let query = Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(2, 2));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..12]);

    // Query a location that intersects with a single packet
    let query = Aabb::new(I16Vec2::new(3, 3), I16Vec2::new(3, 3));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..0, 12..16]);

    // Query a location that doesn't intersect with any packets
    let query = Aabb::new(I16Vec2::new(10, 10), I16Vec2::new(10, 10));
    let result: Vec<_> = bvh.get_in(query).into_iter().collect();
    assert_eq!(result, vec![0..0]);
}
