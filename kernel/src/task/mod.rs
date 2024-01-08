//! Task management

mod context;

use crate::config::*;
use crate::sync::UPSafeCell;
use task::{TaskControlBlock, TaskStatus};
use crate::loader::{
    get_num_app,
    init_app_cx,
};
use context::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

/** AppManager **/
struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    current_task: usize,
    tasks: [TaskControlBlock; MAX_APP_NUM],
}

//TODO
lazy_static! {
    static ref TASK_MANAGER: TaskManager = unsafe {
        let num_app = get_num_app();
        let tasks = [TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::init(),
        }; MAX_APP_NUM];

        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_status = TaskStatus::Ready;
            task.task_cx = TaskContext::goto_restore(
                init_app_cx(i)
            );
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

extern "C" {
    fn __switch(
        current_task_cx_ptr: *mut TaskContext,
        next_task_cx_ptr: *const TaskContext
    );
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.task[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        
        let mut unused_task_cx_ptr = TaskContext::init();
        unsafe {
            __switch(
                &mut unused_task_cx_ptr as *mut TaskContext,
                next_task_cx_ptr
            );
        }  
        
        panic!("Unreachable in TaskManager::run_first_task!");
    }

    //TODOs: schedule
    fn run_next_task(&self) {
        
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn run_next_task() {
    TASK_MANAGER.run_next_task();
}