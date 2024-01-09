//! Task management

mod context;

#[allow(clippy::module_inception)]
mod task;

use lazy_static::*;
use crate::config::*;
use crate::sync::UPSafeCell;
use task::{TaskControlBlock, TaskStatus};
use crate::loader::{
    get_num_app,
    init_app_cx
};
use context::TaskContext;
use crate::sbi::shutdown;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

/** AppManager **/
pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    current_task: usize,
    tasks: [TaskControlBlock; MAX_APP_NUM],
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
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
        let task0 = &mut inner.tasks[0];
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

    /** schedule **/
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
            //back to user
        } else {
            println!("All applications completed!");
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