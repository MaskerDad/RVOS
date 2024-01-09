//! Load apps to memory

use crate::config::*;
use crate::trap::TrapContext;
use core::arch::asm;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
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

    pub fn push_context(&self, cx: TrapContext) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_cx_ptr = cx;
        }
        trap_cx_ptr as usize
    }
}

pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(
        TrapContext::app_init_context(
            get_app_base(app_id),
            USER_STACK[app_id].get_sp(),
        )
    )
}

/** load all apps **/
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe {
        (_num_app as usize as *const usize).read_volatile()
    }
}

fn get_app_base(i: usize) -> usize {
    APP_BASE_ADDRESS + i * APP_SIZE_LIMIT
}

pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }
    println!("[kernel] Loading all apps...");
    
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)  
    };
    
    unsafe {
        asm!("fence.i");
    }
    //begin load apps
    for i in 0..num_app {
        let app_base = get_app_base(i);
        //clear memory
        (app_base..app_base + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe {
                (addr as *mut u8).write_volatile(0)
            });
        //copy src to dst
        let app_src = unsafe {
            core::slice::from_raw_parts(
                app_start[i] as *const u8,
                app_start[i + 1] - app_start[i]
            )
        };
        let app_dst = unsafe {
            core::slice::from_raw_parts_mut(
                app_base as *mut u8,
                app_src.len()
            )
        };
        app_dst.copy_from_slice(app_src);
    }
}