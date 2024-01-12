//! RVOS memory management

use crate::test::mm_test::heap_test;
mod heap_allocator;

pub fn init() {
    heap_allocator::init_heap();
    heap_test();
}