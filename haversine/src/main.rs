fn main() {
    let test_time = 10_000_000;
    println!("\n---WriteToAllBytes--");
    haversine::repetition_test_write_bytes(test_time);
    println!("\n\n---MOVAllBytes--");
    haversine::repetition_test_write_mov(test_time);
    println!("\n\n---NOPAllBytes--");
    haversine::repetition_test_write_nop(test_time);
    println!("\n\n---CMPAllBytes--");
    haversine::repetition_test_write_cmp(test_time);
    println!("\n\n---DecAllBytes--");
    haversine::repetition_test_write_dec(test_time);
}
