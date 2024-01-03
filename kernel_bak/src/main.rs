//! The main module of RVOS

#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

#[macro_use]
mod console;
mod lang_items;
mod sbi;

global_asm!(include_str!("entry.asm"));

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    clear_bss();
    println!("[kernel] Hello, RVOS!");
    //rvos_logo();
    sbi::shutdown(false)
}

/*
pub fn rvos_logo() {
    println!("						  _____                    _____                   _______                   _____          	        ");
	println!("						 /\    \                  /\    \                 /::\    \                 /\    \         	        ");
	println!("						/::\    \                /::\____\               /::::\    \               /::\    \        	        ");
	println!("					   /::::\    \              /:::/    /              /::::::\    \             /::::\    \       	        ");
	println!("					  /::::::\    \            /:::/    /              /::::::::\    \           /::::::\    \      	        ");
	println!("					 /:::/\:::\    \          /:::/    /              /:::/~~\:::\    \         /:::/\:::\    \     	        ");
	println!("					/:::/__\:::\    \        /:::/____/              /:::/    \:::\    \       /:::/__\:::\    \    	        ");
	println!("				   /::::\   \:::\    \       |::|    |              /:::/    / \:::\    \      \:::\   \:::\    \   	        ");
	println!("				  /::::::\   \:::\    \      |::|    |     _____   /:::/____/   \:::\____\   ___\:::\   \:::\    \  	        ");
	println!("				 /:::/\:::\   \:::\____\     |::|    |    /\    \ |:::|    |     |:::|    | /\   \:::\   \:::\    \ 	        ");
	println!("				/:::/  \:::\   \:::|    |    |::|    |   /::\____\|:::|____|     |:::|    |/::\   \:::\   \:::\____\	        ");
	println!("				\::/   |::::\  /:::|____|    |::|    |  /:::/    / \:::\    \   /:::/    / \:::\   \:::\   \::/    /	        ");
	println!("				 \/____|:::::\/:::/    /     |::|    | /:::/    /   \:::\    \ /:::/    /   \:::\   \:::\   \/____/ 	        ");
	println!("					   |:::::::::/    /      |::|____|/:::/    /     \:::\    /:::/    /     \:::\   \:::\    \     	        ");
	println!("					   |::|\::::/    /       |:::::::::::/    /       \:::\__/:::/    /       \:::\   \:::\____\    	        ");
	println!("					   |::| \::/____/        \::::::::::/____/         \::::::::/    /         \:::\  /:::/    /    	        ");
	println!("					   |::|  ~|               ~~~~~~~~~~                \::::::/    /           \:::\/:::/    /     	        ");
	println!("					   |::|   |                                          \::::/    /             \::::::/    /      	        ");
	println!("					   \::|   |                                           \::/____/               \::::/    /       	        ");
	println!("					    \:|   |                                            ~~                      \::/    /        	        ");
	println!("					     \|___|                                                                     \/____/         	        ");
							                                                                                                    
}
*/