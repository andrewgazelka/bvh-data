use bvh::{Bvh, Data, Point};
use glam::I16Vec2;

struct LocalPacket<'a> {
    origin: I16Vec2,
    data: &'a [u8],
}

impl<'a> Point for LocalPacket<'a> {
    // todo: test returning val vs ref
    fn point(&self) -> I16Vec2 {
        self.origin
    }
}

impl<'a> Data<'a> for LocalPacket<'a> {
    type Unit = u8;

    fn data(&self) -> &'a [u8] {
        self.data
    }
}

#[test]
fn test_local_packet() {
    let data = [1, 2, 3, 4];
    let packet = LocalPacket {
        origin: I16Vec2::new(1, 2),
        data: &data,
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
        LocalPacket {
            origin: I16Vec2::new(0, 0),
            data: &data1,
        },
        LocalPacket {
            origin: I16Vec2::new(1, 1),
            data: &data2,
        },
        LocalPacket {
            origin: I16Vec2::new(2, 2),
            data: &data3,
        },
        LocalPacket {
            origin: I16Vec2::new(3, 3),
            data: &data4,
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
