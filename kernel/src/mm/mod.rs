//! RVOS memory management

use crate::test::mm_test::{
    heap_test, 
    frame_allocator_test
};
mod heap_allocator;
mod frame_allocator;
mod address;

pub use address::{
    PhysAddr, PhysPageNum,
    VirtAddr, VirtPageNum
};
pub use frame_allocator::{
    frame_alloc,
    FrameTracker
};

pub fn init() {
    heap_allocator::init_heap();
    heap_test();

    frame_allocator::init_frame_allocator();
    frame_allocator_test();
}