use haversine::math_functions::AvxPackedDoubles;

fn main() {
    let v1 = AvxPackedDoubles::from([10.0, 2.0, 3.0, 4.0]);
    let v2 = AvxPackedDoubles::from([2.0, 3.0, 4.0, 5.0]);
    let mask = haversine::math_functions::simd_gt_mask(&v1, &v2);
    println!("{mask:b}");
}
