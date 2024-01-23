//! Task management
mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use lazy_static::*;
use crate::sync::UPSafeCell;
use task::{TaskControlBlock, TaskStatus};
use crate::loader::{
    get_num_app,
    get_app_data,
};
use context::TaskContext;
use crate::sbi::shutdown;
use core::arch::global_asm;
use alloc::vec::Vec;
use switch::__switch;
use crate::trap::TrapContext;

global_asm!(include_str!("switch.S"));

/* AppManager */
pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    current_task: usize,
    tasks: Vec<TaskControlBlock>,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("[kernel] initialize TASK_MANAGER");
        let num_app = get_num_app();
        println!("[kernel] num_app = {}", num_app);

        let mut tasks: Vec<TaskControlBlock> = Vec::new();

        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
  
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    current_task: 0,
                    tasks,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);    
        let mut unused_task_cx_ptr = TaskContext::zero_init();
        
        unsafe {
            __switch(
                &mut unused_task_cx_ptr as *mut TaskContext,
                next_task_cx_ptr
            );
        }  
        
        panic!("Unreachable in TaskManager::run_first_task!");
    }

    /* 
        [schedule-control-flow]
        Switch current `Running` task to the task we have found, or there is
        no `Ready` task and we can exit with all applications completed.
    */
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            inner.tasks[next].task_status = TaskStatus::Running;
            let cur = inner.current_task;
            inner.current_task = next;
            
            let cur_task_cx_ptr = &mut inner.tasks[cur].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);

            unsafe {
                __switch(cur_task_cx_ptr, next_task_cx_ptr);
            }
            /* bbbbbback to user! */
        } else {
            println!("[kernel] All applications completed!");
            shutdown(false);
        }
    }

    //only return the first `Ready` task in task list
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // [0, 1, 2, 3, 4]
        //     c  n
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }
    
    fn mark_current_suspend(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Exited;
    }

    //get `satp` of current task
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    //get [&mut TrapContext] of current task by trap_cx_ppn
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspend() {
    TASK_MANAGER.mark_current_suspend();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspend();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}