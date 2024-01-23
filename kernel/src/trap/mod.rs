//! Trap Control and Handling
mod context;
pub use context::TrapContext;

use crate::syscall::syscall;
use core::arch::{asm, global_asm};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap, Interrupt},
    stval, stvec, sie
};
use crate::task::{
    current_trap_cx,
    current_user_token,
    exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::set_next_trigger;
use crate::config::{
    TRAMPOLINE,
    TRAP_CONTEXT,
};

global_asm!(include_str!("trap.S"));

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

//initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct)
    };
}

#[no_mangle]
// Unimplement: traps/interrupts/exceptions from kernel mode
// TODO: after I/O device supported
fn trap_from_kernel() -> ! {
    panic!("Not supported by RVOS: a trap from kernel!");
}

#[no_mangle]
pub fn trap_handler() -> ! {
    //set `stvec` for RVOS with S-Mode
    set_kernel_trap_entry();
       
    let trap_cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // the length of `ecall` is 4 byte
            trap_cx.sepc += 4;
            trap_cx.x[10] = syscall(trap_cx.x[17], [trap_cx.x[10], trap_cx.x[11], trap_cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            exit_current_and_run_next();    
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "RVOS does not support traps: {:?}, stval = {:#x}",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    
    let trap_cx_ptr: usize = TRAP_CONTEXT;
    let user_satp = current_user_token();
    
    /*
        We set stvec to the TRAMPOLINE address of the springboard page shared
        between the kernel and the application address space instead of the
        __alltraps address seen by the compiler when linking.
        
        This is because when paging mode is enabled, the kernel can only actually
        retrieve the __alltraps and __restore assembler code from virtual addresses
        on the TRAMPOLINE page.
    */
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",                 // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,              // a0 = va of Trap Context
            in("a1") user_satp,                // a1 = pa of usr page table
            options(noreturn)
        );
    }
}