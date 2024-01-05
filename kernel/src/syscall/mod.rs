//! Implementation of syscalls
//! trap_handler->scause() == UserEnvCall

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        //SYSCALL_EXIT => sys_exit();
        _ => panic!("RVOS does not support syscall_id: {}", syscall_id),
    }
}
