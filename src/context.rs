use std::{alloc::Layout, ffi::c_void, ptr::null_mut};

use neonucleus::ffi::{nn_Alloc, nn_Clock, nn_Context, nn_Rng, nn_noMutex};

#[link(wasm_import_module = "neoweb_utils")]
unsafe extern "C" {
    #[link_name = "get_time"]
    fn _get_time() -> f64;
}

pub fn get_time() -> f64 {
    unsafe { _get_time() }
}

unsafe extern "C" fn alloc(_userdata: *mut c_void, ptr: *mut c_void, old_size: usize, new_size: usize, _extra: *mut c_void) -> *mut c_void {
    const NULL: *mut c_void = null_mut();
    match (ptr, old_size, new_size) {
        (NULL, 0, n) => {
            unsafe { std::alloc::alloc(Layout::from_size_align_unchecked(n, 16)) }.cast()
        },
        (ptr, size, 0) => {
            unsafe { std::alloc::dealloc(ptr.cast(), Layout::from_size_align_unchecked(size, 16)) };
            NULL
        }
        (ptr, size, new_size) => {
            unsafe { std::alloc::realloc(ptr.cast(), Layout::from_size_align_unchecked(size, 16), new_size) }.cast()
        }
    }
}
unsafe extern "C" fn time(_: *mut c_void) -> f64 {
    get_time()
}

static mut SEED: u64 = 0;

// random func: https://git.musl-libc.org/cgit/musl/tree/src/prng/rand.c
unsafe extern "C" fn random(_: *mut c_void) -> usize {
    unsafe { SEED = 6364136223846793005 * SEED + 1 };
	(unsafe { SEED } >> 33) as usize
}

pub fn init_random() {
    unsafe { SEED = get_time().to_bits() };
}
pub fn get_context() -> nn_Context {
    nn_Context {
        allocator: nn_Alloc {
            userdata: null_mut(),
            proc_: Some(alloc),
        },
        lockManager: unsafe { nn_noMutex() },
        clock: nn_Clock {
            userdata: null_mut(),
            proc_: Some(time),
        },
        rng: nn_Rng {
            userdata: null_mut(),
            maximum: usize::MAX,
            proc_: Some(random),
        },
    }
}
