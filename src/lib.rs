use core::slice;
use std::{alloc::{alloc, dealloc, Layout}, ptr::null_mut};

use neonucleus::ffi::{nn_addEEPROM, nn_addFileSystem, nn_addGPU, nn_addKeyboard, nn_addScreen, nn_architecture, nn_computer, nn_eepromControl, nn_filesystemControl, nn_getContext, nn_getError, nn_gpuControl, nn_loadCoreComponentTables, nn_mountKeyboard, nn_newComputer, nn_newScreen, nn_setDepth, nn_tickComputer, nn_veepromOptions, nn_vfilesystemImageNode, nn_vfilesystemOptions, nn_volatileEEPROM, nn_volatileFilesystem};

use crate::context::{get_context, init_random};
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
    init_random();
    let universe = unsafe { neonucleus::ffi::nn_newUniverse(get_context()) };
    assert_ne!(universe, null_mut());
    unsafe { nn_loadCoreComponentTables(universe) };
    let computer = unsafe { nn_newComputer(universe, c"test".as_ptr().cast_mut(), (&ARCH_TABLE as *const nn_architecture).cast_mut(), null_mut(), 1024 * 1024 * 64, 16) };
    assert_ne!(computer, null_mut());
    unsafe { COMPUTER = computer };

    let mut ctx = get_context();
    let screen = unsafe { nn_newScreen(&raw mut ctx, 80, 25, 24, 16, 256) };
    unsafe { nn_setDepth(screen, 8) }; // looks cool
    unsafe { nn_addKeyboard(screen, c"browser keyboard".as_ptr().cast_mut()) };
    unsafe { nn_mountKeyboard(computer, c"browser keyboard".as_ptr().cast_mut(), 2) };
    unsafe { nn_addScreen(computer, null_mut(), 2, screen) };

    let mut gpu_ctrl = nn_gpuControl {
        totalVRAM: 16*1024,
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

    unsafe { nn_addGPU(computer, null_mut(), 3, &raw mut gpu_ctrl) };
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_block(size: i32) -> *mut u8 {
    assert!(size > 0);
    unsafe { alloc(Layout::from_size_align(size as usize, 1).unwrap()) }
}
/// # Safety
/// code and data must point to code_size and data_size bytes of memory allocated with alloc_block
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

    unsafe { dealloc(code, Layout::from_size_align(code_size as usize, 1).unwrap()) };
    if !data.is_null() && data_size > 0 {
        unsafe { dealloc(data, Layout::from_size_align(data_size as usize, 1).unwrap()) };
    }
}

/// # Safety
/// ptr must point to size bytes of memory allocated with alloc_block
#[unsafe(no_mangle)]
pub unsafe extern "C" fn load_vfs(ptr: *mut u8, size: i32) {
    let computer = unsafe { COMPUTER };
    assert_ne!(computer, null_mut());

    let root_len = u32::from_be_bytes(unsafe { ptr.cast::<[u8; 4]>().read() });
    let bytes = unsafe { slice::from_raw_parts(ptr.add(4), size as usize - 4) };
    let entries = neotar::read_entries_recursive(bytes);

    let mut image = Vec::with_capacity(entries.len());

    for entry in entries.iter() {
        let name = entry.name.as_ptr();
        match entry.entry {
            neotar::EntryInner::File(contents) => {
                image.push(nn_vfilesystemImageNode {
                    name,
                    data: contents.as_ptr().cast(),
                    len: contents.len(),
                });
            },
            neotar::EntryInner::Directory(len) => {
                image.push(nn_vfilesystemImageNode {
                    name,
                    data: null_mut(),
                    len: len as usize,
                });
            },
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

    let vfs = unsafe { nn_volatileFilesystem(&raw mut ctx, opts, nn_filesystemControl {
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
    }) };
    unsafe { nn_addFileSystem(computer, null_mut(), 1, vfs) };
    unsafe { dealloc(ptr, Layout::from_size_align(size as usize, 1).unwrap()) };
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
