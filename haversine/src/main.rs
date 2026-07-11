use std::path::Path;

fn main() {
    let filepath = Path::new("data/haversine.json");
    let result = haversine::parse_json_and_calculate_haversine(filepath);
    println!(
        "{}",
        result.iter().sum::<f64>() / (result.len() as f64).sqrt()
    );
}
