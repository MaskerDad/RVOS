//! Task management

mod context;

use crate::config::*;
use crate::sync::UPSafeCell;
use task::{TaskControlBlock, TaskStatus};
use crate::loader::get_num_app;
use context::TaskContext;

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
            //TODO: task.task_cx =
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
        
        panic!("Unreachable in TaskManager::run_first_task!");
    }

    fn run_next_task(&self) {
        
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn run_next_task() {
    TASK_MANAGER.run_next_task();
}