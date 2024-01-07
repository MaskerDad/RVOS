//! Load apps to memory

use crate::config::*;
use crate::trap::TrapContext;
use core::arch::asm;

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

static USER_STACK: [UserStack; MAX_APP_NUM]
= [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

static KERNEL_STACK: [KernelStack; MAX_APP_NUM]
= [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    //TODO
    pub fn push_context(&self, cx: TrapContext) -> &mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe {
            cx_ptr.as_mut().unwrap()
        }
    }
}

//TODO
pub fn load_apps() {
    
    println!("[kernel] Loading app_{}", app_id);
    
    // from {.data: app_start_(app_id)} to APP_BASE_ADDRESS
    core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
    let app_src = core::slice::from_raw_parts(
        self.app_start[app_id] as *const u8,
        self.app_start[app_id + 1] - self.app_start[app_id],  
    );
    let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
    app_dst.copy_from_slice(app_src);
    
    asm!("fence.i");
}
