use std::path::Path;

fn main() {
    let filepath = Path::new("data/haversine.json");
    haversine::parse_and_sum_profiled(filepath);
}
