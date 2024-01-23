//! The unit of task management:[TaskControlBlock]
use super::context::TaskContext;
use crate::config::{
    kernel_stack_position,
    TRAP_CONTEXT,
};
use crate::trap::{
    TrapContext,
    trap_handler,
};
use crate::mm::{
    VirtAddr,
    PhysPageNum,
    MemorySet,
    MapPermission,
    KERNEL_SPACE,
};

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        //install the map_area of kernel stack
        let (kstack_bottom, kstack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().install_framed_area(
            kstack_bottom.into(),
            kstack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_status = TaskStatus::Ready;    
        //constructing the task context
        let task_cx = TaskContext::goto_trap_return(kstack_top);
        
        //get trap_cx_ppn
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        
        //createing TCB
        let task_control_block = Self {
            task_status,
            task_cx,
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };

        //initialize the trap_context of the first running task
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        );
        
        task_control_block
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}