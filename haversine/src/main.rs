use std::path::Path;
fn main() {
    let filepath = Path::new("data/haversine.json");
    haversine::compare_haversine_simd(filepath);
}
