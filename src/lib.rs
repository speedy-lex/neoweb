#![feature(panic_payload_as_str)]

use core::slice;
use std::{
    alloc::{alloc, dealloc, Layout}, ptr::null_mut
};

use neonucleus::ffi::{
    nn_addEEPROM, nn_addFileSystem, nn_addGPU, nn_addKeyboard, nn_addScreen, nn_architecture, nn_computer, nn_eepromControl, nn_filesystemControl, nn_getComputerUserData, nn_getError, nn_getPixel, nn_getTemperature, nn_gpuControl, nn_isOn, nn_isOverheating, nn_loadCoreComponentTables, nn_mountKeyboard, nn_newComputer, nn_newScreen, nn_pushSignal, nn_removeHeat, nn_screen, nn_setDepth, nn_setEnergyInfo, nn_tickComputer, nn_universe, nn_value, nn_values_cstring, nn_values_integer, nn_veepromOptions, nn_vfilesystemImageNode, nn_vfilesystemOptions, nn_volatileEEPROM, nn_volatileFilesystem, NN_STATE_BLACKOUT, NN_STATE_CLOSING, NN_STATE_REPEAT, NN_STATE_SWITCH
};
use neotar::Deserialize;

use crate::arch::ARCH_TABLE;
use crate::context::{get_context, init_random};

mod arch;
mod context;

#[link(wasm_import_module = "neoweb_console")]
unsafe extern "C" {
    #[link_name = "set_row"]
    fn _set_row(id: i32, y: i32, ptr: *const u8, len: usize);
}
#[link(wasm_import_module = "neoweb_utils")]
unsafe extern "C" {
    fn debug_log(ptr: *const i8);
    fn debug_error(ptr: *const i8);
}

fn set_row(id: usize, y: usize, str: &str) {
    unsafe { _set_row(id as i32, y as i32, str.as_ptr(), str.len()) };
}

static mut UNIVERSE: *mut nn_universe = null_mut();

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    #[cfg(debug_assertions)]
    std::panic::set_hook(Box::new(panic_hook));

    init_random();
    let universe = unsafe { neonucleus::ffi::nn_newUniverse(get_context()) };
    assert_ne!(universe, null_mut());
    unsafe { nn_loadCoreComponentTables(universe) };
    unsafe { UNIVERSE = universe };
}

#[unsafe(no_mangle)]
pub extern "C" fn new_computer() -> *mut nn_computer {
    let universe = unsafe { UNIVERSE };
    assert_ne!(universe, null_mut());
    let computer = unsafe {
        nn_newComputer(
            universe,
            c"test".as_ptr().cast_mut(),
            (&ARCH_TABLE as *const nn_architecture).cast_mut(),
            Box::into_raw(Box::new(1_i32)).cast(),
            1024 * 1024 * 2,
            16,
        )
    };
    assert_ne!(computer, null_mut());

    let mut gpu_ctrl: nn_gpuControl = nn_gpuControl {
        totalVRAM: 16 * 1024,
        maximumBufferCount: 64,
        defaultBufferWidth: 80,
        defaultBufferHeight: 25,
        screenCopyPerTick: 8.0,
        screenFillPerTick: 16.0,
        screenSetsPerTick: 32.0,
        bitbltPerTick: 8.0,
        heatPerPixelChange: 0.00005,
        heatPerPixelReset: 0.00001,
        heatPerVRAMChange: 0.00000015,
        energyPerPixelChange: 0.05,
        energyPerPixelReset: 0.01,
        energyPerVRAMChange: 0.0015,
    };

    unsafe { nn_addGPU(computer, null_mut(), 0, &raw mut gpu_ctrl) };
    computer
}

/// # Safety
/// computer must be valid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn new_screen(computer: *mut nn_computer, add_kb: bool) -> *mut nn_screen {
    assert_ne!(computer, null_mut());
    let slot: &mut i32 = unsafe { &mut *(nn_getComputerUserData(computer).cast()) };

    let mut ctx = get_context();
    let screen = unsafe { nn_newScreen(&raw mut ctx, 80, 25, 24, 16, 256) };
    assert_ne!(screen, null_mut());

    unsafe { nn_setDepth(screen, 8) };
    if add_kb {
        unsafe { nn_addKeyboard(screen, c"browser keyboard".as_ptr().cast_mut()) };
        unsafe { nn_mountKeyboard(computer, c"browser keyboard".as_ptr().cast_mut(), *slot) };
    }
    unsafe { nn_addScreen(computer, null_mut(), *slot, screen) };

    *slot += 1;

    screen
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_block(size: i32) -> *mut u8 {
    assert!(size > 0);
    unsafe { alloc(Layout::from_size_align(size as usize, 1).unwrap()) }
}
/// # Safety
/// code and data must point to code_size and data_size bytes of memory allocated with alloc_block
#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_eeprom(
    computer: *mut nn_computer,
    code: *mut u8,
    code_size: i32,
    code_len: i32,
    data: *mut u8,
    data_size: i32,
    data_len: i32,
) {
    assert_ne!(computer, null_mut());
    let slot: &mut i32 = unsafe { &mut *(nn_getComputerUserData(computer).cast()) };
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

    let generic_eeprom = unsafe {
        nn_volatileEEPROM(
            &raw mut ctx,
            opts,
            nn_eepromControl {
                readHeatPerByte: 0.0015,
                writeHeatPerByte: 0.03,
                readEnergyCostPerByte: 0.001,
                writeEnergyCostPerByte: 0.05,
                bytesReadPerTick: 32768.0,
                bytesWrittenPerTick: 4096.0,
            },
        )
    };

    unsafe { nn_addEEPROM(computer, null_mut(), *slot, generic_eeprom) };
    *slot += 1;

    unsafe {
        dealloc(
            code,
            Layout::from_size_align(code_size as usize, 1).unwrap(),
        )
    };
    if !data.is_null() && data_size > 0 {
        unsafe {
            dealloc(
                data,
                Layout::from_size_align(data_size as usize, 1).unwrap(),
            )
        };
    }
}

/// # Safety
/// ptr must point to size bytes of memory allocated with alloc_block
#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_vfs(computer: *mut nn_computer, ptr: *mut u8, size: i32) {
    assert_ne!(computer, null_mut());
    let slot: &mut i32 = unsafe { &mut *(nn_getComputerUserData(computer).cast()) };

    let bytes = unsafe { slice::from_raw_parts(ptr, size as usize) };
    let file = neotar::File::read(bytes).0;
    file.sanity_check();
    let section = file.sections[0];
    let (entries, root_len) = neotar::files::read_entries_recursive(&section);

    let mut image = Vec::with_capacity(entries.len());

    for entry in entries.iter() {
        let name = entry.name.as_ptr();
        match entry.entry {
            neotar::files::EntryInner::File(contents) => {
                image.push(nn_vfilesystemImageNode {
                    name,
                    data: contents.as_ptr().cast(),
                    len: contents.len(),
                });
            }
            neotar::files::EntryInner::Directory(len) => {
                image.push(nn_vfilesystemImageNode {
                    name,
                    data: null_mut(),
                    len: len as usize,
                });
            }
        }
    }

    let opts = nn_vfilesystemOptions {
        creationTime: 0,
        maxDirEntries: 64,
        capacity: 1024 * 1024,
        isReadOnly: false,
        label: [0; 128],
        labelLen: 0,
        image: image.as_mut_ptr(),
        rootEntriesInImage: root_len as usize,
    };

    let mut ctx = get_context();

    let vfs = unsafe {
        nn_volatileFilesystem(
            &raw mut ctx,
            opts,
            nn_filesystemControl {
                readBytesPerTick: 65536.0,
                writeBytesPerTick: 32768.0,
                removeFilesPerTick: 16.0,
                createFilesPerTick: 16.0,

                readHeatPerByte: 0.00000015,
                writeHeatPerByte: 0.0000015,
                removeHeat: 0.035,
                createHeat: 0.045,

                readEnergyPerByte: 0.0015,
                writeEnergyPerByte: 0.0035,
                removeEnergy: 0.135,
                createEnergy: 0.325,
            },
        )
    };
    unsafe { nn_addFileSystem(computer, null_mut(), *slot, vfs) };
    *slot += 1;
    unsafe { dealloc(ptr, Layout::from_size_align(size as usize, 1).unwrap()) };
}

/// # Safety
/// Perhaps
#[unsafe(no_mangle)]
pub unsafe extern "C" fn on_key(computer: *mut nn_computer, char: i32, code: i32, released: bool) {
    assert_ne!(computer, null_mut());

    unsafe {
        let mut values: [nn_value; 5] = [
            nn_values_cstring(if released {
                c"key_up".as_ptr()
            } else {
                c"key_down".as_ptr()
            }),
            nn_values_cstring(c"browser keyboard".as_ptr()),
            nn_values_integer(char as i64),
            nn_values_integer(code as i64),
            nn_values_cstring(c"USER".as_ptr()),
        ];
        nn_pushSignal(computer, values.as_mut_ptr(), 5);
    }
}

/// # Safety
/// computer must be valid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tick(computer: *mut nn_computer) {
    assert_ne!(computer, null_mut());

    unsafe { nn_setEnergyInfo(computer, f64::INFINITY, f64::INFINITY) };
        
    let heat = unsafe { nn_getTemperature(computer) };
    unsafe { nn_removeHeat(computer, heat) };
    
    if unsafe { nn_isOverheating(computer) } {
        unsafe { debug_error(c"overheating".as_ptr()) };
        return;
    }

    let state = unsafe { nn_tickComputer(computer) };
    match state {
        NN_STATE_SWITCH => {
            unsafe { debug_log(c"state switch".as_ptr()) };
        }
        NN_STATE_BLACKOUT => {
            unsafe { debug_log(c"blackout".as_ptr()) };
        }
        NN_STATE_REPEAT => {
            unsafe { debug_log(c"reboot".as_ptr()) };
        }
        NN_STATE_CLOSING => {
            unsafe { debug_log(c"shutdown".as_ptr()) };
        }
        _ => {}
    }
    let error = unsafe { nn_getError(computer) };
    if !error.is_null() {
        unsafe { debug_error(error.cast()) }
        unreachable!();
    }
}

/// # Safety
/// screen must be valid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn update_screen(screen: *mut nn_screen, id: usize) {
    assert_ne!(screen, null_mut());

    if unsafe { nn_isOn(screen) } {
        let mut str = String::with_capacity(2 * 80); // most chars are ascii, so most of the time no realloc and worst case 1 realloc
        for y in 0..25 {
            str.clear();
            for x in 0..80 {
                let pixel = unsafe { nn_getPixel(screen, x, y as i32) };
                str.push(char::from_u32(pixel.codepoint).unwrap_or_default());
            }
            set_row(id, y, &str);
        }
    }
}

#[cfg(debug_assertions)]
fn panic_hook(info: &std::panic::PanicHookInfo) {
    unsafe { debug_error(c"PANIC".as_ptr()) };
    if let Some(location) = info.location() {
        let mut str = format!("{location:?}");
        str.push('\0');
        let bytes = str.into_bytes();
        unsafe { debug_error(bytes.as_ptr().cast()) };
    }
    if let Some(mut str) = info.payload_as_str().map(str::to_owned) {
        str.push('\0');
        let bytes = str.into_bytes();
        unsafe { debug_error(bytes.as_ptr().cast()) };
    }
}
