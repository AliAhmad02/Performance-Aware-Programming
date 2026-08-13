fn main() {
    let test_time = 10_000_000;
    // Region sizes that fit at each cache level: L1, L2, L3, memory
    let region_sizes = [1 << 12, 1 << 17, 1 << 21, 1 << 27];

    let alignments = [0, 1, 2, 3, 15, 16, 17, 31, 32, 33, 48, 63, 64, 65];

    for size in region_sizes {
        for alignment in alignments {
            haversine::repetition_test_double_loop_read_32x8(test_time, size, alignment);
            println!();
        }

        println!("----------------")
    }
}
