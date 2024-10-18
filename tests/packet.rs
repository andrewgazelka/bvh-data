use bvh::{Aabb, Bvh, Data, Point};
use glam::I16Vec2;

struct ChunkWithPackets<'a> {
    location: I16Vec2,
    packets_data: &'a [u8],
}

impl Default for ChunkWithPackets<'_> {
    fn default() -> Self {
        Self {
            location: I16Vec2::new(0, 0),
            packets_data: &[],
        }
    }
}

impl Point for ChunkWithPackets<'_> {
    // todo: test returning val vs ref
    fn point(&self) -> I16Vec2 {
        self.location
    }
}

impl Data for ChunkWithPackets<'_> {
    type Unit = u8;

    fn data(&self) -> &[u8] {
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
fn test_build_bvh_with_empty_input() {
    let data: Vec<ChunkWithPackets> = vec![];

    let bvh = Bvh::build(data, 0);

    assert_eq!(bvh.elements().len(), 0);

    assert_eq!(bvh.get_closest_slice(I16Vec2::new(0, 0)), None);

    assert_eq!(
        bvh.get_in(Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(100, 100)))
            .len(),
        0
    );
}

#[test]
fn test_build_bvh_1_packet() {
    let data = [1];
    let packet = ChunkWithPackets {
        location: I16Vec2::new(1, 2),
        packets_data: &data,
    };

    let bvh = Bvh::build(vec![packet], 1);

    let print = bvh.print();

    assert_eq!(print, "01	Leaf([1, 2] -> 0)");

    assert_eq!(bvh.elements().len(), 1);
    assert_eq!(bvh.elements(), &[1]);

    assert_eq!(
        bvh.get_closest_slice(I16Vec2::new(1, 2)),
        Some([1].as_slice())
    );

    assert_eq!(
        bvh.get_closest_slice(I16Vec2::new(4, 4)),
        Some([1].as_slice())
    );

    assert_eq!(
        bvh.get_in(Aabb::new(I16Vec2::new(0, 0), I16Vec2::new(100, 100)))
            .len(),
        1
    );
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

    let bvh = Bvh::build(input, size_hint);

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
fn test_query_single_packet() {
    let data = [1, 2, 3, 4];
    let packet = ChunkWithPackets {
        location: I16Vec2::new(1, 2),
        packets_data: &data,
    };

    let input = vec![packet];
    let size_hint = input.len() * 4;

    let bvh = Bvh::build(input, size_hint);

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

    let bvh = Bvh::build(input, size_hint);

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

    // we can make bytes BVH
    let _bvh = bvh.into_bytes();
}
