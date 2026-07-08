use std::fs;
use std::path::Path;

const PERFORMANCE_AWARE_PATH: &str = "/home/ali/Rust_code/Performance_Aware_Programming";

fn main() {
    let path = Path::new(PERFORMANCE_AWARE_PATH).join("Listings/listing_52");
    let binary = fs::read(path).unwrap();
    let simulate = true;
    if simulate {
        simulator_8086::simulate(binary);
    } else {
        simulator_8086::decode_binary_and_print(binary);
    }
}
