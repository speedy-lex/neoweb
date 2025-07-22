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
    for x in 0..80 {
        for y in 0..25 {
            set_cell(x, y, 'a');
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() {
    
}
