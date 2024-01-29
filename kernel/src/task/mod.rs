//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod pid;
mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::get_app_data_by_name;
use crate::sbi::shutdown;
use task::{TaskControlBlock, TaskStatus};
pub use manager::{add_task, fetch_task};
use context::TaskContext;
use switch::__switch;
use pid::{pid_alloc, KernelStack, PidHandle};
pub use processor::{
    current_task, take_current_task, current_trap_cx, current_user_token,
    run_tasks, schedule, Processor,
};

use lazy_static::*;
use alloc::sync::Arc;

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

///Add init process to the TASK_MANAGER
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub const INIT_PID: usize = 0;

///Suspend the current `Running` task and run the next task by `fetch_task`
pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Ready;
    let switched_task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    drop(task_inner);

    add_task(task);
    schedule(switched_task_cx_ptr);
}

///Exit the current `Running` task and run the next task by `fetch_task`
pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let pid = task.getpid();
    if pid == INIT_PID {
        println!("[RVOS] INITPROC process exit with exit_code {} ...", exit_code);
        if exit_code != 0 {
            shutdown(true)
        } else {
            shutdown(false)
        }
    }
    
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Zombie;
    task_inner.exit_code = exit_code;
    
    //access initproc TCB for inherit current_task's children
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in task_inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }

    //release current_task partially
    task_inner.children.clear();
    task_inner.memory_set.recycle_data_pages();
    drop(task_inner);
    //drop task manually to maintain rc correctly
    drop(task);
    //switch to the next task
    //we do not have to save current task_context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut TaskContext);
}