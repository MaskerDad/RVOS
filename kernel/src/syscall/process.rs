//! Process management syscalls

use crate::timer::get_time_ms;
use crate::task::{
    add_task, current_task, current_user_token,
    exit_current_and_run_next,
    suspend_current_and_run_next,    
};
use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// get current time
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    //return 0 for children
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    trap_cx.x[10] = 0;
    //return new_pid for parent
    add_task(new_task);
    new_pid as isize
}