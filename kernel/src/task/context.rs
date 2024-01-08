//! Implementation of [TaskContext]

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
    pub fn init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}