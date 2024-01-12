//! Physical page frame allocator: supports the automatic reclamation mechanism

use super::{
    PhysAddr,
    PhysPageNum
};
use core::fmt::{self, Debug, Formatter};
use alloc:vec::Vec;
use lazy_static::*;
use crate::{sync::UPSafeCell, mm::address::PhysPageNum};
use crate::config::MEMORY_END;

/*
    Single frame describe: [FrameTracker]
*/
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        //clear page
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;            
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);   
    }
}

/*
    Global frame allocator:[StackFrameAllocator]
*/
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>, 
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        //validity check
        if self.current <= ppn || 
        self.recycled.iter().any(|&v| v == ppn) {
            panic!("ppn = {:#x} has not been allocated!", ppn);        
        } 
        self.recycled.push(ppn);    
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

/* init frame allocator */
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe {
            UPSafeCell::new(FrameAllocatorImpl::new())
        };
}

/* interfaces for external use */
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysPageNum::from(ekernel as usize).ceil(),
        PhysPageNum::from(MEMORY_END).floor()
    );
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}