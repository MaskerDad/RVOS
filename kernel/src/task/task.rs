//!Implementation of [`TaskControlBlock`]

use super::{pid_alloc, KernelStack, PidHandle};
use super::TaskContext;
use crate::config::TRAP_CONTEXT;
use crate::mm::{MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};
use crate::sync::UPSafeCell;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;
use core::mem;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
pub struct TaskControlBlock {
    //immmutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    //mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.task_status == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self)
        -> RefMut<'_, TaskControlBlockInner>
    {
        self.inner.exclusive_access()
    }
    
    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_sp, entry_point) =
            MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                })
            },
        };

        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize
        );
        task_control_block
    }
    
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut parent_inner = self.inner_exclusive_access();
        //copy user_space
        let memory_set = MemorySet::from_existed_user_space(&parent_inner.memory_set);
        //alloc pid/kernel_stack
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        //create new task_control_block
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let kernel_stack_top = kernel_stack.get_top();
        let task_cx = TaskContext::goto_trap_return(kernel_stack_top);
        let task_status = TaskStatus::Ready;
        let task_control_block = Arc::new(
            TaskControlBlock {
                pid: pid_handle,
                kernel_stack,
                inner: unsafe {
                    UPSafeCell::new(
                        TaskControlBlockInner {
                            trap_cx_ppn,
                            base_size: parent_inner.base_size,
                            task_cx,
                            task_status,
                            memory_set,
                            parent: Some(Arc::downgrade(self)),
                            children: Vec::new(),
                            exit_code: 0,
                        }
                    )
                }
            }
        );
        //add child
        parent_inner.children.push(task_control_block.clone());
        //modify kernel_sp in trap_cx
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        //return
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        //generate new user_space for app's elf_data
        //memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        //update current_task(self)
        let mut task_inner = self.inner_exclusive_access();
        task_inner.memory_set = memory_set;
        task_inner.trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        //initialize trap_cx
        let trap_cx = task_inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize
        );
    }
}