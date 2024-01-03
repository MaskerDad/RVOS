//! For no-std object, we need to impl some items to run it

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        //print
    } else {
        //print
    }
    
}