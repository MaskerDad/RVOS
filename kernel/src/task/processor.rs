//! PROCESSOR

use std::process;

use alloc::sync::Arc;
use lazy_static::*;

use crate::sync::UPSafeCell;
use super::fetch_task;
use super::__switch;
use super::{TaskControlBlock, TaskContext};

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = {
        unsafe {
            UPSafeCell::new(Processor::new())
        }
    };
}
pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    
    pub fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref()
            .map(|task| Arc::clone(task))
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_trap_cx() -> &'static mut TaskContext {
    
}

pub fn current_token() -> usize {
    
}

///Processor fetch and run task
///IDLE_FLOW_OF_CONTROL
pub fn run_tasks() {
    loop {
        let mut process = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = process.get_idle_task_cx_ptr();
            //TODO
        }
    }
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut process = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = process.get_idle_task_cx_ptr();
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
