fn main() {
    let test_time = 10_000_000;
    println!("\n\n---NOP3x1AllBytes--");
    haversine::repetition_test_nop31(test_time);
    println!("\n\n---NOP1x3AllBytes--");
    haversine::repetition_test_nop13(test_time);
    println!("\n\n---NOP1x9AllBytes--");
    haversine::repetition_test_nop19(test_time);
}
