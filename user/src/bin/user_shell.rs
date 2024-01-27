#![no_std]
#![no_main]
#![allow(cippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

use alloc::string::String;
use user_lib::console::getchar;
use user_lib::{fork, exec, waitpid};

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

#[no_mangle]
pub fn main() -> i32 {
    println!("[RVOS] user_shell");
    let mut line = String::new();
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    line.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        if exec(line.as_ptr()) == -1 {
                            println!("The application name is incorrect!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code = 0i32;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!(
                            "user_shell: process {} exited with code {}",
                            pid, exit_code,
                        );
                    }
                    line.clear();                 
                }
                print!(">> ");
            },
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            },
            _ => {
                print!("{}", c as char);
                line.push(c as char);  
            },
        }
    }
}