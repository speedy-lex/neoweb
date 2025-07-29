use std::{alloc::{alloc, Layout}, ptr::null_mut};

use neonucleus::ffi::{nn_addEEPROM, nn_architecture, nn_computer, nn_eepromControl, nn_getError, nn_loadCoreComponentTables, nn_newComputer, nn_tickComputer, nn_veepromOptions, nn_volatileEEPROM};

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

static mut COMPUTER: *mut nn_computer = null_mut();

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    set_cell(0, 0, 0, 'n');
    init_random();
    let universe = unsafe { neonucleus::ffi::nn_newUniverse(get_context()) };
    assert_ne!(universe, null_mut());
    unsafe { nn_loadCoreComponentTables(universe) };
    let computer = unsafe { nn_newComputer(universe, c"test".as_ptr().cast_mut(), (&ARCH_TABLE as *const nn_architecture).cast_mut(), null_mut(), 1024 * 1024 * 64, 16) };
    assert_ne!(computer, null_mut());
    unsafe { COMPUTER = computer };
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_eeprom(size: i32) -> *mut u8 {
    assert!(size <= 1024 * 1024); // no huge allocs
    assert!(size > 0);
    unsafe { alloc(Layout::from_size_align(size as usize, 1).unwrap()) }
}
/// # Safety
/// code and data must point to code_size and data_size bytes of memory allocated with alloc_eeprom
#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_eeprom(code: *mut u8, code_size: i32, code_len: i32, data: *mut u8, data_size: i32, data_len: i32) {
    let computer = unsafe { COMPUTER };
    assert_ne!(computer, null_mut());
    assert_ne!(code, null_mut());
    
    let opts = nn_veepromOptions {
        code: code.cast(),
        len: code_len as usize,
        size: code_size as usize,
        data: data.cast(),
        dataLen: data_len as usize,
        dataSize: data_size as usize,
        label: [0; 128],
        labelLen: 0,
        isReadOnly: false,
    };

    let mut ctx = get_context();

    let generic_eeprom = unsafe { nn_volatileEEPROM(&raw mut ctx, opts, nn_eepromControl {
        readHeatPerByte: 0.0015,
        writeHeatPerByte: 0.03,
        readEnergyCostPerByte: 0.001,
        writeEnergyCostPerByte: 0.05,
        bytesReadPerTick: 32768.0,
        bytesWrittenPerTick: 4096.0,
    }) };

    unsafe { nn_addEEPROM(computer, null_mut(), 0, generic_eeprom) };
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() {
    let computer = unsafe { COMPUTER };
    assert_ne!(computer, null_mut());
    let txt = format!("{}", unsafe { nn_tickComputer(computer) });
    for (i, ch) in txt.chars().enumerate() {
        set_cell(0, i, 0, ch);
    }
    let mut error = unsafe { nn_getError(computer) };
    if !error.is_null() {
        let mut i = 0;
        while unsafe { *error } != 0 {
            set_cell(0, i, 1, unsafe { *error } as u8 as char);
            error = unsafe { error.add(1) };
            i += 1;
        }
        unreachable!();
    }
}
