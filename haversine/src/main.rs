use std::path::Path;

fn main() {
    let filepath = Path::new("data/haversine.json");
    haversine::repetition_test_read(filepath, 10_000_000);
}
