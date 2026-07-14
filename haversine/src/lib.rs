use rand::RngExt;
use std::arch::x86_64::_rdtsc;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const EARTH_RADIUS: f64 = 6372.8;
const OS_TIMER_FREQ: u64 = 1_000_000;
static PROFILE_RESULTS: Mutex<Vec<ProfileResult>> = Mutex::new(Vec::new());

#[macro_export]
macro_rules! time {
    ($label:literal, $($tt:tt)+) => {
        {
            let time_start = read_cpu_timer();
            let return_value = {$( $tt )+};
            let time_end = read_cpu_timer();
            PROFILE_RESULTS.lock().unwrap().push(ProfileResult {time_start, time_end, label: $label});
            return_value
        }
    };
}

pub fn parse_and_sum_profiled_auto(filepath: &Path) {
    let string = time!(
        "Allocate JSON string",
        fs::read_to_string(filepath).unwrap()
    );
    let values: Vec<f64> = time! {
        "Parse JSON",
        string
            .split([':', ',', '}'])
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let n_pairs = (values.len() / 4) as f64;
    let sum: f64 = time! {
        "Calculate sum",
        values
            .chunks_exact(4)
            .map(|x| haversine(x[0], x[1], x[2], x[3]))
            .sum::<f64>()
            / n_pairs.sqrt()
    };

    time! {
        "Print results",
        println!("Number of pairs: {n_pairs:.0}");
        println!("Sum(haversine)/sqrt(N): {sum:.16}");
    };
    print_profile_results(PROFILE_RESULTS.lock().unwrap().get(..).unwrap());
}

fn print_profile_results(results: &[ProfileResult]) {
    let cpu_time_total = (results[results.len() - 1].time_end - results[0].time_start) as f64;
    let cpu_freq = estimate_cpu_timer_freq();
    let real_time_total_ms = 1000.0 * cpu_time_total / (cpu_freq as f64);

    println!(
        "Total time {:.4} ms (CPU freq: {})",
        real_time_total_ms, cpu_freq
    );

    for result in results {
        let cpu_time = result.timediff();
        let cpu_time_pct = (cpu_time as f64) / cpu_time_total * 100.0;
        println!("  {}: {}, ({:.2}%)", result.label, cpu_time, cpu_time_pct);
    }
}

pub fn parse_and_sum_profiled(filepath: &Path) {
    let cpu_begin = read_cpu_timer();
    let string = fs::read_to_string(filepath).unwrap();
    let cpu_allocated = read_cpu_timer();
    let values: Vec<f64> = string
        .split([':', ',', '}'])
        .filter_map(|s| s.parse().ok())
        .collect();
    let n_pairs = (values.len() / 4) as f64;
    let cpu_parsed = read_cpu_timer();
    let sum: f64 = values
        .chunks_exact(4)
        .map(|x| haversine(x[0], x[1], x[2], x[3]))
        .sum::<f64>()
        / n_pairs.sqrt();
    let cpu_calculate_and_sum = read_cpu_timer();

    println!("Number of pairs: {n_pairs:.0}");
    println!("Sum(haversine)/sqrt(N): {sum:.16}");

    let cpu_results_printed = read_cpu_timer();
    let cpu_freq = estimate_cpu_timer_freq();
    let cpu_time_total = (cpu_results_printed - cpu_begin) as f64;

    println!(
        "Total time: {:.4} ms (CPU freq: {})",
        1000.0 * cpu_time_total / (cpu_freq as f64),
        cpu_freq
    );

    println!(
        "  Allocate JSON string: {}, ({:.2}%)",
        cpu_allocated - cpu_begin,
        ((cpu_allocated - cpu_begin) as f64) / cpu_time_total * 100.0
    );

    println!(
        "  Parse JSON: {}, ({:.2}%)",
        cpu_parsed - cpu_allocated,
        ((cpu_parsed - cpu_allocated) as f64) / cpu_time_total * 100.0
    );

    println!(
        "  Calculate sum: {}, ({:.2}%)",
        cpu_calculate_and_sum - cpu_parsed,
        ((cpu_calculate_and_sum - cpu_parsed) as f64) / cpu_time_total * 100.0
    );

    println!(
        "  Print results: {}, ({:.2}%)",
        cpu_results_printed - cpu_calculate_and_sum,
        ((cpu_results_printed - cpu_calculate_and_sum) as f64) / cpu_time_total * 100.0
    );
}
fn estimate_cpu_timer_freq() -> u64 {
    let miliseconds_to_wait = 100;
    let cpu_time_start = read_cpu_timer();
    let os_time_start = read_os_timer();
    let mut os_time_end;
    let mut os_time_elapsed = 0;
    // We need to wait for 100 ms which is 100_000 microseconds
    let os_wait_time = OS_TIMER_FREQ * miliseconds_to_wait / 1000;

    while os_time_elapsed < os_wait_time {
        os_time_end = read_os_timer();
        os_time_elapsed = os_time_end - os_time_start;
    }

    let cpu_time_end = read_cpu_timer();
    let cpu_time_elapsed = cpu_time_end - cpu_time_start;

    if os_time_elapsed > 0 {
        // Number of CPU ticks per second (the division gives per microsecond
        // and the factor million converts back to seconds)
        OS_TIMER_FREQ * cpu_time_elapsed / os_time_elapsed
    } else {
        0
    }
}

// Read time in microseconds
fn read_os_timer() -> u64 {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_secs() * OS_TIMER_FREQ + duration.subsec_micros() as u64
}

// Read time in CPU units (number of elapsed "cpu ticks")
fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub fn parse_json_and_calculate_haversine(filepath: &Path) -> Vec<f64> {
    let string = fs::read_to_string(filepath).unwrap();
    let values: Vec<f64> = string
        .split([':', ',', '}'])
        .filter_map(|s| s.parse().ok())
        .collect();
    values
        .chunks_exact(4)
        .map(|x| haversine(x[0], x[1], x[2], x[3]))
        .collect()
}

pub fn generate_haversine_json(n: u32, filepath: &Path) {
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

struct ProfileResult {
    time_start: u64,
    time_end: u64,
    label: &'static str,
}

impl ProfileResult {
    fn timediff(&self) -> u64 {
        self.time_end - self.time_start
    }
}
