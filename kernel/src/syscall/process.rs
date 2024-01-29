//! Process management syscalls

use crate::mm::{translated_str, translate_refmut};
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
    exit_current_and_run_next(exit_code);
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

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
/// Else return child_pid.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();    
    //return -1
    if !task_inner
        .children.iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
    }
    //return pid or -2
    let pair = task_inner.children.iter().enumerate()
        .find(|(_, p)|{
            p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())   
        });
    if let Some((idx, _)) = pair {
        /*
            It completely reclaims all the resources it occupies, including the kernel
            stack and its PID, the physical page frames of the page tables in its
            application address space, and so on.
        */
        let child = task_inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        //confirm that child will be deallocated after removing form children list
        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive_access().exit_code;
        *translate_refmut(task_inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else{
        -2
    }
}