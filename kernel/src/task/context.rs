//! Implementation of [TaskContext]
use crate::trap::trap_return;

/* 
    callee saved:
    x1 - ra
    x2 - sp
    x8~x9/x18~x27 - s0~s11
*/
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    //The initial context in which the task first runs
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}