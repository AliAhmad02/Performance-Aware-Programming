use rand::RngExt;
use std::fmt::Write;
use std::fs;
use std::path::Path;

const EARTH_RADIUS: f64 = 6372.8;

pub fn generate_haversine_json(n: u32) {
    // We assume that each line is around 110 bytes
    let mut writer = String::with_capacity(n as usize * 110);
    let iter = generate_haversine_data(n);
    let mut first = true;

    writeln!(&mut writer, "{{\"pairs\":[").unwrap();

    for (((x0, y0), x1), y1) in iter {
        if !first {
            writeln!(&mut writer, ",").unwrap();
        }
        first = false;

        write!(
            &mut writer,
            "{{\"x0\":{x0:.16}, \"y0\":{y0:.16}, \"x1\":{x1:.16}, \"y1\":{y1:.16}}}"
        )
        .unwrap();
    }

    writeln!(&mut writer, "\n]}}").unwrap();

    let filepath = Path::new("data/haversine.json");
    fs::write(filepath, writer).unwrap();
}

fn generate_haversine_data(n: u32) -> impl Iterator<Item = (((f64, f64), f64), f64)> {
    let (lon1, lat1) = sample_lon_lat_uniform(n);
    let (lon2, lat2) = sample_lon_lat_uniform(n);

    lon1.into_iter().zip(lat1).zip(lon2).zip(lat2)
}

pub fn test_haversine(n: u32) -> f64 {
    let (lon1, lat1) = sample_lon_lat_uniform(n);
    let (lon2, lat2) = sample_lon_lat_uniform(n);

    lon1.into_iter()
        .zip(lat1)
        .zip(lon2)
        .zip(lat2)
        .map(|(((a, b), c), d)| haversine(a, b, c, d))
        .sum::<f64>()
        / f64::from(n).sqrt()
}

fn haversine(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    // Note: Compiling with optimizations causes the powi(2) calls to
    // just be two multiplies, so it's fine to use it. Proof:
    // https://godbolt.org/z/o6oad7o7e
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);

    2.0 * a.sqrt().asin() * EARTH_RADIUS
}

/* Sampling uniformly on a sphere is not as simple as sampling
uniformly in zenith θ and azimuth φ. To understand this,
consider an infinitesimal area element on the surface of
a sphere in spherical coordinates
dA = r^2 sin(θ) dθ dφ
Clearly, it is not uniform; the change in area introduced by
a change in θ depends on sin(θ), so we will overpopulate regions
where the change is small. The fix here is simple; let
u = cos(θ) => dθ = - du / sin(θ) => dA = r^2 du dφ
which is uniform! Thus, we can sample uniformly on a sphere
simply by sampling φ uniformly and cos(θ) uniformly between
-1 and 1 and after that we just do arccos(u) to get θ.
Final detail: We describe the position on Earth using two
coordinates latitude θ and longitude φ. If you look at
https://www.math.ksu.edu/~dbski/writings/haversine.pdf
on page 2 and compare the transformations to cartesian
coordinates to the ones from standard spherical coordinates
we notice that the only difference is cos(θ) -> sin(θ),
so for lat/long coords we just sample sin(θ) instead. */
fn sample_lon_lat_uniform(n: u32) -> (Vec<f64>, Vec<f64>) {
    // This is what Casey calls x
    let longitude = sample_values_in_range(-180.0, 180.0, n);
    // This is what Casey calls y
    let latitude: Vec<f64> = sample_values_in_range(-1.0, 1.0, n)
        .iter()
        .map(|x| x.asin().to_degrees())
        .collect();
    (longitude, latitude)
}

fn sample_values_in_range(low: f64, high: f64, n: u32) -> Vec<f64> {
    let mut rng = rand::rng();
    let uniform = rand::distr::Uniform::new_inclusive(low, high).unwrap();
    let mut values = Vec::with_capacity(n as usize);
    for _ in 0..n {
        values.push(rng.sample(uniform));
    }

    values
}
