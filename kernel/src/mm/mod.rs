//! RVOS memory management

mod heap_allocator;
mod frame_allocator;
mod address;
mod memory_set;
mod page_table;

/*
use crate::test::mm_test::{
    heap_test, 
    frame_allocator_test,
    remap_test,
};
*/

pub use address::{
    PhysAddr, PhysPageNum,
    VirtAddr, VirtPageNum,
    VPNRange, StepByOne,
};
pub use frame_allocator::{
    frame_alloc,
    FrameTracker
};
pub use page_table::{
    PageTable,
    PageTableEntry,
    PTEFlags,
    translated_byte_buffer,
};
pub use memory_set::{
    MemorySet,
    MapPermission,
    KERNEL_SPACE,
};

pub fn init() {
    //heap_init
    heap_allocator::init_heap();
    //heap_test();

    //frame_allocator init
    frame_allocator::init_frame_allocator();
    //frame_allocator_test();

    //KERNEL_SPACE init
    KERNEL_SPACE.exclusive_access().activate();
    //remap_test();
}