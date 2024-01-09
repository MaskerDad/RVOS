//! Constants used in rCore

/* USER/KERNEL STACK */
pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

/* APP */
pub const MAX_APP_NUM: usize = 10;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;

/* TIMER */
pub const CLOCK_FREQ: usize = 12500000;