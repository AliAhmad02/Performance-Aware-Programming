fn main() {
    let test_time = 10_000_000;
    for i in 10..30 {
        haversine::repetition_test_read_32x8(test_time, i);
        println!();
    }
}
