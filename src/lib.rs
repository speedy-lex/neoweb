use crate::context::{get_context, init_random};

mod context;

#[link(wasm_import_module = "neoweb_console")]
unsafe extern "C" {
    #[link_name = "set_cell"]
    fn _set_cell(x: i32, y: i32, ch: i32);
}

fn set_cell(x: usize, y: usize, ch: char) {
    unsafe { _set_cell(x as i32, y as i32, ch as i32) };
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    init_random();
    let universe = unsafe { neonucleus::ffi::nn_newUniverse(get_context()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() {
    
}
