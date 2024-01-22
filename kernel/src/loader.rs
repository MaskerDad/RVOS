/*!
    The Loader's job is to fetch the ELF data of an application from
    a specified location in kernel.data because rCore supports virtual
    memory, the data of different apps need not be preallocated into
    different physical address Spaces. The physical page frame allocator
    determines which physical page frames the data of these apps are
    actually loaded on when the virtual address space is created.
*/
pub fn get_num_app() -> usize() {
    extern "C" {
        fn _num_app();
    }
    unsafe {
        (_num_app as usize as *const usize).read_volatile()
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    let num_app = get_num_app();
    assert!(app_id < num_app);
    
    extern "C" {
        fn _num_app();
    }
    let num_app_str = _num_app as usize as *const usize;
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_str.add(1), num_app + 1)
    };
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]
        )
    }
}