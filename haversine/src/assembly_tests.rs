use crate::*;

unsafe extern "C" {
    fn TemporalWrite(
        read_pointer: *const BytesAligned32,
        write_pointer: *mut BytesAligned32,
        inner_count: usize,
        outer_count: usize,
    );
    fn NonTemporalWrite(
        read_pointer: *const BytesAligned32,
        write_pointer: *mut BytesAligned32,
        inner_count: usize,
        outer_count: usize,
    );
    fn DoubleLoopRead_32x8(outer_count: usize, pointer: *const u8, inner_count: usize);
    fn Read_32x8(count: usize, pointer: *const u8, mask: usize);
    fn Read_4x2(count: usize, pointer: *const u8);
    fn Read_8x2(count: usize, pointer: *const u8);
    fn Read_16x2(count: usize, pointer: *const u8);
    fn Read_32x2(count: usize, pointer: *const u8);
    fn Write_x1(count: usize, pointer: *const u8);
    fn Write_x2(count: usize, pointer: *const u8);
    fn Write_x3(count: usize, pointer: *const u8);
    fn Write_x4(count: usize, pointer: *const u8);
    fn Read_x1(count: usize, pointer: *const u8);
    fn Read_x2(count: usize, pointer: *const u8);
    fn Read_x3(count: usize, pointer: *const u8);
    fn Read_x4(count: usize, pointer: *const u8);
    fn MOVAllBytesASM(count: usize, pointer: *mut u8);
    fn NOPAllBytesASM(count: usize, pointer: *mut u8);
    fn CMPAllBytesASM(count: usize, pointer: *mut u8);
    fn DECAllBytesASM(count: usize, pointer: *mut u8);
    fn NOP3x1AllBytes(count: usize, pointer: *mut u8);
    fn NOP1x3AllBytes(count: usize, pointer: *mut u8);
    fn NOP1x9AllBytes(count: usize, pointer: *mut u8);

}

// A vector inherits its alignment from its datatype, so we can
// create a 32-byte aligned vector by storing a 32-byte aligned
// struct that holds 32 bytes of data
#[repr(C, align(32))]
#[derive(Copy, Clone)]
struct BytesAligned32 {
    values: [u8; 32],
}

impl Default for BytesAligned32 {
    fn default() -> Self {
        BytesAligned32 { values: [1; 32] }
    }
}

pub fn repetition_test_compare_temp_nontemp(test_time: u64, region_size: usize) {
    let buffer_size = 1024 * 1024 * 1024;
    let outer_count = buffer_size / region_size;
    let inner_count = region_size / 256;
    let total_size = outer_count * region_size;

    let mut tester_temp = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        total_size,
    );
    let mut tester_nontemp = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        total_size,
    );

    println!("Temporal results:");

    let mut read_buffer = vec![BytesAligned32::default(); region_size / 32];
    let mut write_buffer = vec![BytesAligned32::default(); buffer_size / 32];
    let mut elapsed_total = 0;
    while elapsed_total < test_time {
        let start_os_time = read_os_timer();
        tester_temp.start_measurements();
        unsafe {
            TemporalWrite(
                read_buffer.as_ptr(),
                write_buffer.as_mut_ptr(),
                inner_count,
                outer_count,
            );
        }
        let reset_timer = tester_temp.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester_temp.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester_temp.print_maximum();
    tester_temp.print_average();

    println!("\nNon-Temporal results:");

    read_buffer = vec![BytesAligned32::default(); region_size / 32];
    write_buffer = vec![BytesAligned32::default(); buffer_size / 32];
    elapsed_total = 0;
    while elapsed_total < test_time {
        let start_os_time = read_os_timer();
        tester_nontemp.start_measurements();
        unsafe {
            NonTemporalWrite(
                read_buffer.as_ptr(),
                write_buffer.as_mut_ptr(),
                inner_count,
                outer_count,
            );
        }
        let reset_timer = tester_nontemp.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester_nontemp.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester_nontemp.print_maximum();
    tester_nontemp.print_average();
}

pub fn repetition_test_double_loop_read_32x8(test_time: u64, region_size: usize, alignment: usize) {
    let buffer_size = 1024 * 1024 * 1024;
    let outer_count = buffer_size / region_size;
    let inner_count = region_size / 256;
    let total_size = outer_count * region_size;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        total_size,
    );

    let mut elapsed_total = 0;
    let buffer = vec![1; buffer_size];

    while elapsed_total < test_time {
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            DoubleLoopRead_32x8(
                outer_count,
                buffer.as_ptr().wrapping_add(alignment),
                inner_count,
            );
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_32x8(test_time: u64, mask_pow: u64) {
    let buffer_size = 1 << 30;
    let region_size = 1 << mask_pow;
    let mask = region_size - 1;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        buffer_size,
    );

    let mut elapsed_total = 0;
    let buffer = vec![1; buffer_size];

    while elapsed_total < test_time {
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_32x8(buffer_size, buffer.as_ptr(), mask);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_32x2(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_32x2(num_bytes, buffer.as_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_16x2(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_16x2(num_bytes, buffer.as_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_8x2(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_8x2(num_bytes, buffer.as_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_4x2(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_4x2(num_bytes, buffer.as_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_x4(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Write_x4(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_x3(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Write_x3(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_x2(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Write_x2(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_x1(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Write_x1(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_x4(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_x4(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_x3(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_x3(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_x2(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_x2(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_read_x1(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let buffer = Box::new(10u8);
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            Read_x1(num_bytes, &*buffer as *const u8);
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_nop19(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            NOP1x9AllBytes(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_nop13(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            NOP1x3AllBytes(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_nop31(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            NOP3x1AllBytes(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_dec(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            DECAllBytesASM(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_cmp(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            CMPAllBytesASM(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_nop(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            NOPAllBytesASM(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}

pub fn repetition_test_write_mov(test_time: u64) {
    let num_bytes = 1024 * 1024;
    let mut tester = RepetitionTest::build(
        vec![
            Measurement::CpuTime(CpuTime::new()),
            Measurement::PageFaults(PageFaults::new()),
        ],
        num_bytes,
    );

    let mut elapsed_total = 0;

    while elapsed_total < test_time {
        let mut buffer = vec![0; num_bytes];
        let start_os_time = read_os_timer();
        tester.start_measurements();
        unsafe {
            MOVAllBytesASM(num_bytes, buffer.as_mut_ptr());
        }
        let reset_timer = tester.stop_measurements();
        let elapsed_os_time = read_os_timer() - start_os_time;
        if reset_timer {
            elapsed_total = 0;
            tester.print_minimum();
        } else {
            elapsed_total += elapsed_os_time;
        }
    }

    tester.print_maximum();
    tester.print_average();
}
