use std::fs;
use std::path::Path;

const PERFORMANCE_AWARE_PATH: &str = "/home/ali/Rust_code/Performance_Aware_Programming";

fn main() {
    let path = Path::new(PERFORMANCE_AWARE_PATH).join("Listings/listing_46");
    let binary = fs::read(path).unwrap();
    simulator_8086::simulate(binary);
}
