use haversine::{asin_ce, cos_quarter, get_max_diff_funcs, sin_half, sin_quarter, sqrt_ce};
use std::f64::consts::PI;

fn main() {
    let sin_half_max = get_max_diff_funcs(f64::sin, sin_half, -PI, PI, 1_000_000);
    let sin_quarter_max = get_max_diff_funcs(f64::sin, sin_quarter, -PI, PI, 1_000_000);
    let cos_max = get_max_diff_funcs(f64::cos, cos_quarter, -PI / 2.0, PI / 2.0, 1_000_000);
    let asin_max = get_max_diff_funcs(f64::asin, asin_ce, 0.0, 1.0, 1_000_000);
    let sqrt_max = get_max_diff_funcs(f64::sqrt, sqrt_ce, 0.0, 1.0, 1_000_000);

    println!("Sine half approximation maximum difference: {sin_half_max}");
    println!("Sine quarter approximation maximum difference: {sin_quarter_max}");
    println!("Cosine quarter approximation maximum difference: {cos_max}");
    println!("Asin maximum difference: {asin_max}");
    println!("Sqrt maximum difference: {sqrt_max}");

}
