//! Load apps and back to user

use crate::trap::TrapContext;
use crate::sbi::shutdown;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

/** AppManager **/
struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }

    pub fn print_app_info(&self) {
        
    }

    pub fn load_app(&self, app_id: usize) {
        // all apps completed
        if app_id >= self.num_app {
            println!("All applications completed!");
            shutdown(false);
        }
        
        println!("[kernel] Loading app_{}", app_id);
        
        // from {.data: app_start_(app_id)} to APP_BASE_ADDRESS
        core::slice::from_raw_parts(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],  
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src);
        
        asm!("fence.i");
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new(
            {
                extern "C" {
                    fn _num_app();
                }
                let num_app_ptr = _num_app as usize as *const usize;
                let num_app = num_app_ptr.read_volatile();
                let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
                let app_start_raw: &[usize] = 
                    core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
                app_start[..=num_app].copy_from_slice(app_start_raw);
                AppManager {
                    num_app,
                    current_app: 0,
                    app_start,
                }
            }
        )
    };
}

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    app_manager.load_app(current_app);
    //TODO

    panic!("Unreachable in batch::run_next_app");
}