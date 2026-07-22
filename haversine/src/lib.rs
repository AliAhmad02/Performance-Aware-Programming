use core::f64;
use rand::RngExt;
use std::arch::{asm, x86_64::_rdtsc};
use std::fmt::{Display, Write as Write1};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const EARTH_RADIUS: f64 = 6372.8;
const OS_TIMER_FREQ: u64 = 1_000_000;
// Mutex allows us to mutate the immutable static value.
// Lazylock allows us to pre-allocate the memory for the
// vector at runtime (typically for a static everything
// has to happen at compile time)
static PROFILE_RESULTS: LazyLock<Mutex<Vec<ProfileResult>>> =
    LazyLock::new(|| Mutex::new(Vec::with_capacity(100)));

#[macro_export]
macro_rules! time_simple {
    ($($tt:tt)+) => {
        {
            let time_start = read_cpu_timer();
            {$( $tt )+};
            let time_end = read_cpu_timer();
            time_end - time_start
        }
    };
}

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

/*
The page fault numbers were not making sense (I was not getting
4kB per page fault. Turns out this is because of an optimization
in Linux called Transparent Hugepages that allows to allocate
2 MB chunks instead of just 4kB. If I disable this, the results
make perfect sense; I get 4kB/fault if I allocate inside the
loop and 0 faults if I allocate outside the loop
*/
pub fn repetition_test_read(filepath: &Path, test_time: u64) {
    let mut buffer = fs::read(filepath).unwrap();
    let num_bytes = buffer.len();
    //     let num_bytes = 108429019;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let start_os_time = read_os_timer();
        let mut file = fs::File::open(filepath).unwrap();

        //        let mut buffer = vec![0; tester.bytes];
        tester.start_measurements();
        file.read_exact(&mut buffer).unwrap();
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
        } else {
            elapsed_total += elapsed_os_time;
        }
        tester.count += 1;

        print!("\x1B[2J\x1B[H");
        print!("{}", tester);
        io::stdout().flush().unwrap();
    }
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

fn close(fd: i32) {
    const SYSCALL_CLOSE: usize = 3;
    let return_code: i32;

    unsafe {
        asm!(
            "syscall",
            in("rax") SYSCALL_CLOSE,
            in("rdi") fd,
            // syscall clobbers rcx and r11
            out("rcx") _,
            out("r11") _,
            lateout("rax") return_code,
        )
    }

    assert!(return_code >= 0);
}

fn read(fd: i32, pointer: &mut u64, size: usize) {
    const SYSCALL_READ: usize = 0;
    let return_code: i32;

    unsafe {
        asm!(
            "syscall",
            in("rax") SYSCALL_READ,
            in("rdi") fd,
            in("rsi") pointer,
            in("rdx") size,
            // syscall clobbers rcx and r11
            out("rcx") _,
            out("r11") _,
            lateout("rax") return_code,
        )
    }

    assert!(return_code >= 0);
}

fn ioctl(fd: i32, code: usize, pointer: usize) {
    const SYSCALL_IOCTL: usize = 16;
    let return_code: i32;

    unsafe {
        asm!(
            "syscall",
            in("rax") SYSCALL_IOCTL,
            in("rdi") fd,
            in("rsi") code,
            in("rdx") pointer,
            // syscall clobbers rcx and r11
            out("rcx") _,
            out("r11") _,
            lateout("rax") return_code,
        )
    }

    assert!(return_code >= 0);
}

struct RepetitionTest {
    measurements: Vec<Measurement>,
    results: Vec<RepetitionResult>,
    count: u64,
    bytes: usize,
}

impl Display for RepetitionTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cpu_freq = estimate_cpu_timer_freq() as f64;
        let cpu_to_ms = 1000.0 / (cpu_freq);
        let cpu_to_s = 1.0 / cpu_freq;
        let byte_to_gb = 1.0 / (1024.0 * 1024.0 * 1024.0);
        let mut min_string = "Min: ".to_string();
        let mut max_string = "Max: ".to_string();
        let mut avg_string = "Avg: ".to_string();

        for (result, measurement) in self.results.iter().zip(&self.measurements) {
            let average = self.average(result);
            match measurement {
                Measurement::CpuTime(_) => {
                    min_string.push_str(&format!(
                        "{} ({:.3} ms) {:.3} gb/s ",
                        result.min,
                        result.min as f64 * cpu_to_ms,
                        (self.bytes as f64 * byte_to_gb) / (result.min as f64 * cpu_to_s),
                    ));
                    max_string.push_str(&format!(
                        "{} ({:.3} ms) {:.3} gb/s ",
                        result.max,
                        result.max as f64 * cpu_to_ms,
                        (self.bytes as f64 * byte_to_gb) / (result.max as f64 * cpu_to_s),
                    ));
                    avg_string.push_str(&format!(
                        "{:.3} ({:.3} ms) {:.3} gb/s ",
                        result.total as f64 / self.count as f64,
                        average * cpu_to_ms,
                        (self.bytes as f64 * byte_to_gb) / (average * cpu_to_s),
                    ));
                }
                Measurement::PageFaults(_) => {
                    min_string.push_str(&format!(
                        "PF: {} ({:.3}kb/fault) ",
                        result.min,
                        self.bytes as f64 / (result.min as f64 * 1024.0),
                    ));
                    max_string.push_str(&format!(
                        "PF: {} ({:.3}kb/fault) ",
                        result.max,
                        self.bytes as f64 / (result.max as f64 * 1024.0),
                    ));
                    avg_string.push_str(&format!(
                        "PF: {:.3} ({:.3}kb/fault) ",
                        result.total as f64 / self.count as f64,
                        self.bytes as f64 / (average * 1024.0),
                    ));
                }
            }
        }

        write!(f, "{}\n{}\n{}", min_string, max_string, avg_string)
    }
}

impl RepetitionTest {
    fn build(measurements: Vec<Measurement>, bytes: usize) -> Self {
        Self {
            results: vec![
                RepetitionResult {
                    min: u64::MAX,
                    max: 0,
                    total: 0
                };
                measurements.len()
            ],
            measurements,
            count: 0,
            bytes,
        }
    }

    fn start_measurements(&mut self) {
        self.measurements
            .iter_mut()
            .for_each(|m| m.start_measurement());
    }

    fn stop_measurements(&mut self) -> bool {
        let mut reset_timer = false;
        for (result, measurement) in self.results.iter_mut().zip(&mut self.measurements) {
            measurement.stop_measurement();
            let result_value = measurement.get_result();

            if result_value < result.min {
                result.min = result_value;
                if let Measurement::CpuTime(_) = measurement {
                    reset_timer = true;
                }
            } else if result_value > result.max {
                result.max = result_value;
            }
            result.total += result_value;
        }
        reset_timer
    }

    fn average(&self, result: &RepetitionResult) -> f64 {
        result.total as f64 / self.count as f64
    }
}

#[derive(Clone)]
struct RepetitionResult {
    min: u64,
    max: u64,
    total: u64,
}

enum Measurement {
    CpuTime(CpuTime),
    PageFaults(PageFaults),
}

impl Measurement {
    fn start_measurement(&mut self) {
        match self {
            Self::CpuTime(m) => m.start_measurement(),
            Self::PageFaults(m) => m.start_measurement(),
        }
    }

    fn stop_measurement(&mut self) {
        match self {
            Self::CpuTime(m) => m.stop_measurement(),
            Self::PageFaults(m) => m.stop_measurement(),
        }
    }

    fn get_result(&self) -> u64 {
        match self {
            Self::CpuTime(m) => m.end - m.start,
            Self::PageFaults(m) => m.faults,
        }
    }
}

struct CpuTime {
    start: u64,
    end: u64,
}

impl CpuTime {
    fn new() -> Self {
        CpuTime { start: 0, end: 0 }
    }

    fn start_measurement(&mut self) {
        self.start = read_cpu_timer();
    }

    fn stop_measurement(&mut self) {
        self.end = read_cpu_timer();
    }
}

struct PageFaults {
    faults: u64,
    file_descriptor: i32,
}

impl Drop for PageFaults {
    fn drop(&mut self) {
        close(self.file_descriptor);
    }
}

impl PageFaults {
    fn new() -> Self {
        let perf_event = PerfEventAttr {
            r#type: 1,
            size: size_of::<PerfEventAttr>() as u32,
            config: 2,
            // 1 is equivalent to disabled=1
            bit_flags: 1,
            ..Default::default()
        };

        const SYSCALL_PERF_EVENT_OPEN: usize = 298;

        let file_descriptor: i32;
        unsafe {
            asm!(
                "syscall",
                in("rax") SYSCALL_PERF_EVENT_OPEN,
                in("rdi") &perf_event,
                in("rsi") 0,
                in("rdx") -1,
                in("r10") -1,
                in("r8") 0,
                // syscall clobbers rcx and r11
                out("rcx") _,
                out("r11") _,
                // Return pointer to filehandle
                lateout("rax") file_descriptor,
            )
        }

        assert!(file_descriptor >= 0);

        Self {
            faults: 0,
            file_descriptor,
        }
    }

    fn start_measurement(&mut self) {
        const IOCTL_RESET: usize = 9219;
        const IOCTL_ENABLE: usize = 9216;
        ioctl(self.file_descriptor, IOCTL_RESET, 0);
        ioctl(self.file_descriptor, IOCTL_ENABLE, 0);
    }

    fn stop_measurement(&mut self) {
        const IOCTL_DISABLE: usize = 9217;
        ioctl(self.file_descriptor, IOCTL_DISABLE, 0);
        read(self.file_descriptor, &mut self.faults, size_of::<u64>());
    }
}

#[derive(Default)]
#[repr(C)]
struct PerfEventAttr {
    r#type: u32,
    size: u32,
    config: u64,
    sample_period: u64,
    sample_type: u64,
    read_format: u64,
    bit_flags: u64,
    wakeup_events: u32,
    bp_type: u32,
    bp_addr: u64,
    bp_len: u64,
    branch_sample_type: u64,
    sample_regs_user: u64,
    sample_stack_user: u32,
    clockid: i32,
    sample_regs_intr: u64,
    aux_watermark: u32,
    sample_max_stack: u16,
    __reserved_2: u16,
    aux_sample_size: u32,
    aux_action: u32,
    sig_data: u64,
    config3: u64,
    config4: u64,
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
