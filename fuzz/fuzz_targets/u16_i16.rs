#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: i32| {
    let i32_min = i16::MIN as i64;
    let naive_result = if (data as i64) - i32_min;
    let naive_result = u32::try_from(naive_result).unwrap();
    
    let bvh_result = bvh::add_half_max_and_convert(data);
    
    assert_eq!(naive_result, bvh_result, "Mismatch found for input: {data}");
});
