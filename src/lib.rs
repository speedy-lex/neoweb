use crate::context::{get_context, init_random};

mod context;

#[link(wasm_import_module = "neoweb_console")]
unsafe extern "C" {
    #[link_name = "set_cell"]
    fn _set_cell(id: i32, x: i32, y: i32, ch: i32);
}

fn set_cell(id: usize, x: usize, y: usize, ch: char) {
    unsafe { _set_cell(id as i32, x as i32, y as i32, ch as i32) };
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    init_random();
    let universe = unsafe { neonucleus::ffi::nn_newUniverse(get_context()) };
    let addr = format!("{universe:p}");
    for (i, ch) in addr.chars().enumerate() {
        set_cell(0, i, 0, ch);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() {
    
}
