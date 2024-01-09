//! Timer-releated

use riscv::register::time;
use crate::config::CLOCK_FREQ;

const MSEC_PER_SEC: usize = 1000;
const TICKS_PER_SEC: usize = 100; //trigger 100 clock interrupts per second

//mtimers
pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_ms() -> usize {
    (time::read() / CLOCK_FREQ) * MSEC_PER_SEC;
}

/*
    [mtime]: Count how many clock cycles of the built-in clock have elapsed since the processor was powered up
    [mtimecmp]: As soon as the value of the counter `mtime` exceeds `mtimecmp`, a clock interrupt is triggered
*/
//mtimecmp
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}