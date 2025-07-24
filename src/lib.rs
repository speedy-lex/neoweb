use std::ptr::null_mut;

use neonucleus::ffi::{nn_architecture, nn_loadCoreComponentTables, nn_newComputer};

use crate::{context::{get_context, init_random}};
use crate::arch::ARCH_TABLE;

mod context;
mod arch;

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
    set_cell(0, 0, 0, 'n');
    init_random();
    let universe = unsafe { neonucleus::ffi::nn_newUniverse(get_context()) };
    assert_ne!(universe, null_mut());
    unsafe { nn_loadCoreComponentTables(universe) };
    let computer = unsafe { nn_newComputer(universe, c"test".as_ptr().cast_mut(), (&ARCH_TABLE as *const nn_architecture).cast_mut(), null_mut(), 1024 * 1024 * 64, 16) };
    assert_ne!(computer, null_mut());
    let txt = format!("{universe:p}");
    for (i, ch) in txt.chars().enumerate() {
        set_cell(0, i, 0, ch);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() {
    
}
