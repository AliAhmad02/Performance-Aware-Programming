fn main() {
    let test_time = 10_000_000;
    let mut region_sizes = [0usize; 64];

    let mut size_delta = 512;
    let mut accumulated_size = 4 * 1024;

    for size in region_sizes.iter_mut() {
        *size = accumulated_size;

        // Increase the delta every time we hit an even power of two
        if ((accumulated_size - 1) & accumulated_size) == 0 {
            size_delta *= 2;
        }

        accumulated_size += size_delta;
    }

    for size in region_sizes {
        haversine::repetition_test_double_loop_read_32x8(test_time, size);
        println!();
    }

    println!("{:?}", region_sizes);
}
