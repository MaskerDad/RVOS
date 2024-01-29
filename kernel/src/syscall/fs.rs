//! File and filesystem-related syscalls

use crate::mm::translated_byte_buffer;
use crate::task::{current_user_token, suspend_current_and_run_next};
use crate::sbi::console_getchar;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "[RVOS ERROR] only support `len=1` in sys_read!");
            let mut c: usize;
            loop {
                c = console_getchar();
                if c == 0 {
                    suspend_current_and_run_next();
                    continue;
                } else {
                    break;
                }
            }
            let ch: u8 = c as u8;
            let mut buffers_writable = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers_writable[0].as_mut_ptr().write_volatile(ch);
            }
            1
        },
        _ => {
            panic!("[RVOS ERROR] unsupported fd in sys_read!");
        }
    }
}