//! The unit of task management:[TaskControlBlock]
use super::context::TaskContext;
use crate::mm::{
    MemorySet,
    KERNEL_SPACE,
    MapPermission,
};
use crate::config::{
    kernel_stack_position,
    TRAP_CONTEXT,
};

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_self(elf_data);
        let task_status = TaskStatus::Ready;

        //install the map_area of kernel stack
        let (kstack_bottom, kstack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().install_framed_area(
            kstack_bottom.into(),
            kstack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        
        //get trap_cx_ppn


        
        //TODO: constructing the task context
        let task_cx = TaskContext::goto_trap_return(kstack_top);



        le task_control_block = Self {
            task_status,
            task_cx,
            memory_set,
            base_size: user_sp,
        };

        //initialize the trap_context of the first running task
        

        task_control_block
    }
}


