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
//#[allow(clippy::module_inception)]
mod task;

use crate::loader::get_app_data_by_name;
use crate::sbi::shutdown;
use task::{TaskControlBlock, TaskStatus};
use manager::{add_task, fetch_task};
use context::TaskContext;
use switch::__switch;
use pid::{pid_alloc, KernelStack, PidHandle};