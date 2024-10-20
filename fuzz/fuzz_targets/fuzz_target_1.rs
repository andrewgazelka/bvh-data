#![no_main]

use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use std::collections::HashSet;

#[macro_use]
extern crate libfuzzer_sys;
// use libfuzzer_sys::fuzz_target;
use bvh::{Aabb, Bvh, Point};
use glam::I16Vec2;

// Wrapper type for I16Vec2
#[derive(Debug, Clone, Copy)]
struct Vec2Wrapper(I16Vec2);

impl<'a> Arbitrary<'a> for Vec2Wrapper {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let x = i16::arbitrary(u)?;
        let y = i16::arbitrary(u)?;
        Ok(Vec2Wrapper(I16Vec2::new(x, y)))
    }
}

// Wrapper type for Aabb<I16Vec2>
#[derive(Debug, Clone, Copy)]
struct AabbWrapper(Aabb);

impl<'a> Arbitrary<'a> for AabbWrapper {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let min = Vec2Wrapper::arbitrary(u)?.0;
        let max = Vec2Wrapper::arbitrary(u)?.0;
        Ok(AabbWrapper(Aabb::new(min, max)))
    }
}

#[derive(Arbitrary)]
struct FuzzInput {
    chunks: Vec<ChunkWithPackets>,
    query: AabbWrapper,
}

use std::fmt;

impl fmt::Debug for FuzzInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "FuzzInput {{")?;
        writeln!(f, "  chunks: [")?;
        for (i, chunk) in self.chunks.iter().enumerate() {
            writeln!(
                f,
                "    {}: location: {:?}, data: {:?},",
                i, chunk.location.0, chunk.packets_data
            )?;
        }
        writeln!(f, "  ],")?;
        writeln!(
            f,
            "  query: Aabb {{ min: {:?}, max: {:?} }}",
            self.query.0.min, self.query.0.max
        )?;
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, Arbitrary)]
struct ChunkWithPackets {
    location: Vec2Wrapper,
    packets_data: Vec<u8>,
}

impl Point for ChunkWithPackets {
    fn point(&self) -> I16Vec2 {
        self.location.0
    }
}

impl bvh::Data for ChunkWithPackets {
    type Unit = u8;

    fn data(&self) -> &[u8] {
        &self.packets_data
    }
}

fn naive_query(chunks: &[ChunkWithPackets], query: &Aabb) -> HashSet<u8> {
    chunks
        .iter()
        .filter(|chunk| query.contains_point(chunk.location.0))
        .flat_map(|chunk| chunk.packets_data.clone())
        .collect()
}

fn bvh_query(bvh: &Bvh<Vec<u8>>, query: &Aabb) -> HashSet<u8> {
    bvh.get_in_slices(*query)
        .into_iter()
        .flat_map(|slice| slice.to_vec())
        .collect()
}

fuzz_target!(|input: FuzzInput| {
    let bvh = Bvh::build(input.chunks.clone());

    let naive_result = naive_query(&input.chunks, &input.query.0);
    let bvh_result = bvh_query(&bvh, &input.query.0);

    assert_eq!(
        naive_result, bvh_result,
        "Mismatch found for input: {input:?}"
    );
});
