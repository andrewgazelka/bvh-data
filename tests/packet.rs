use bvh::{Bvh, Data, Point};
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

    let mut input = vec![
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
    println!("{}", s);
}
