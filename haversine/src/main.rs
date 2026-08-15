fn main() {
    let test_time = 10_000_000;
    // Region sizes that fit at each cache level: L1, L2, L3, memory
    let region_sizes = [1 << 12, 1 << 17, 1 << 21, 1 << 27];
    for size in region_sizes {
        println!("------------Size={}------------", size);
        haversine::assembly_tests::repetition_test_compare_temp_nontemp(test_time, size);
        println!("\n");
    }
}
