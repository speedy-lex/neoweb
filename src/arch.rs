use std::{ffi::c_void, ptr::{null, null_mut}};

use lua53_sys::{luaL_openlibs, lua_State, lua_close, lua_createtable, lua_getfield, lua_gettop, lua_load, lua_newstate, lua_pushcclosure, lua_pushlightuserdata, lua_resume, lua_setfield, lua_setglobal, lua_settop, lua_tolstring, lua_touserdata, LUA_OK, LUA_REGISTRYINDEX, LUA_YIELD};
use neonucleus::ffi::{nn_alloc, nn_architecture, nn_clearError, nn_computer, nn_dealloc, nn_getAllocator, nn_getComputerMemoryTotal, nn_getUniverse, nn_resize, nn_setCError, nn_setError};

pub const ARCH_TABLE: nn_architecture = nn_architecture {
    userdata: null_mut(),
    archName: c"Lua".as_ptr(),
    setup: Some(setup),
    teardown: Some(teardown),
    getMemoryUsage: Some(get_memory_usage),
    tick: Some(tick),
    serialize: None,
    deserialize: None,
};
const LUA_SANDBOX: &[u8] = include_bytes!("sandbox.lua");

#[repr(C)]
struct State {
    lua: *mut lua_State,
    computer: *mut nn_computer,
    mem_usage: usize,
}

unsafe extern "C" fn lua_alloc(state: *mut c_void, ptr: *mut c_void, old: usize, new: usize) -> *mut c_void {
    let state: &mut State = unsafe { &mut *state.cast() };
    let alloc = unsafe { nn_getAllocator(nn_getUniverse(state.computer)) };
    if new == 0 {
        state.mem_usage -= old;
        unsafe { nn_dealloc(alloc, ptr, old) };
        null_mut()
    } else {
        let mut actual_old = old;
        if ptr.is_null() {
            actual_old = 0;
        }
        if state.mem_usage - actual_old + new > unsafe { nn_getComputerMemoryTotal(state.computer) } {
            return null_mut(); // OOM condition
        }
        state.mem_usage -= actual_old;
        state.mem_usage += new;
        unsafe { nn_resize(alloc, ptr, old, new) }
    }
}
unsafe fn lua_pop(lua: *mut lua_State, n: i32) {
    unsafe { lua_settop(lua, -(n)-1) }
}
unsafe fn get_state(lua: *mut lua_State) -> *mut State {
    unsafe { lua_getfield(lua, LUA_REGISTRYINDEX, c"archPtr".as_ptr()) };
    let state = unsafe { lua_touserdata(lua, -1) };
    unsafe { lua_pop(lua, 1) };
    state.cast()
}
unsafe extern "C" fn computer_clear_error(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };
    unsafe { nn_clearError((*state).computer) };
    0
}
fn load_env(lua: *mut lua_State) {
    unsafe { lua_createtable(lua, 0, 10) };
    let computer = unsafe { lua_gettop(lua) };
    unsafe { lua_pushcclosure(lua, Some(computer_clear_error), 0) };
    unsafe { lua_setfield(lua, computer, c"clearError".as_ptr()) };
    // lua_pushcfunction(lua, testLuaArch_computer_usedMemory);
    // lua_setfield(lua, computer, "usedMemory");
    // lua_pushcfunction(lua, testLuaArch_computer_freeMemory);
    // lua_setfield(lua, computer, "freeMemory");
    // lua_pushcfunction(lua, testLuaArch_computer_totalMemory);
    // lua_setfield(lua, computer, "totalMemory");
    // lua_pushcfunction(lua, testLuaArch_computer_address);
    // lua_setfield(lua, computer, "address");
    // lua_pushcfunction(lua, testLuaArch_computer_tmpAddress);
    // lua_setfield(lua, computer, "tmpAddress");
    // lua_pushcfunction(lua, testLuaArch_computer_uptime);
    // lua_setfield(lua, computer, "uptime");
    // lua_pushcfunction(lua, testLuaArch_computer_beep);
    // lua_setfield(lua, computer, "beep");
    // lua_pushcfunction(lua, testLuaArch_computer_energy);
    // lua_setfield(lua, computer, "energy");
    // lua_pushcfunction(lua, testLuaArch_computer_maxEnergy);
    // lua_setfield(lua, computer, "maxEnergy");
    // lua_pushcfunction(lua, testLuaArch_computer_getArchitecture);
    // lua_setfield(lua, computer, "getArchitecture");
    // lua_pushcfunction(lua, testLuaArch_computer_getArchitectures);
    // lua_setfield(lua, computer, "getArchitectures");
    // lua_pushcfunction(lua, testLuaArch_computer_setArchitecture);
    // lua_setfield(lua, computer, "setArchitecture");
    // lua_pushcfunction(lua, testLuaArch_computer_isOverworked);
    // lua_setfield(lua, computer, "isOverworked");
    // lua_pushcfunction(lua, testLuaArch_computer_isOverheating);
    // lua_setfield(lua, computer, "isOverheating");
    // lua_pushcfunction(lua, testLuaArch_computer_getTemperature);
    // lua_setfield(lua, computer, "getTemperature");
    // lua_pushcfunction(lua, testLuaArch_computer_addHeat);
    // lua_setfield(lua, computer, "addHeat");
    // lua_pushcfunction(lua, testLuaArch_computer_pushSignal);
    // lua_setfield(lua, computer, "pushSignal");
    // lua_pushcfunction(lua, testLuaArch_computer_popSignal);
    // lua_setfield(lua, computer, "popSignal");
    // lua_pushcfunction(lua, testLuaArch_computer_users);
    // lua_setfield(lua, computer, "users");
    // lua_pushcfunction(lua, testLuaArch_computer_getState);
    // lua_setfield(lua, computer, "getState");
    // lua_pushcfunction(lua, testLuaArch_computer_setState);
    // lua_setfield(lua, computer, "setState");
    unsafe { lua_setglobal(lua, c"computer".as_ptr()) };
}
struct ReadData {
    data: &'static [u8],
    i: usize
}
unsafe extern "C" fn one_shot_read(_lua: *mut lua_State, data: *mut c_void, read: *mut usize) -> *const i8 {
    let data: &mut ReadData = unsafe { &mut *data.cast() };
    if data.i >= data.data.len() {
        return null();
    }
    let x = &data.data[data.i..];
    data.i += x.len();
    unsafe { *read = x.len() };
    x.as_ptr().cast()
}
unsafe extern "C" fn setup(computer: *mut nn_computer, _userdata: *mut c_void) -> *mut c_void {
    let alloc = unsafe { nn_getAllocator(nn_getUniverse(computer)) };
    let state = unsafe { nn_alloc(alloc, size_of::<State>()) }.cast::<State>();
    if state.is_null() {
        return null_mut();
    }
    unsafe { (*state).computer = computer };
    unsafe { (*state).mem_usage = 0 };
    let lua = unsafe { lua_newstate(Some(lua_alloc), state.cast()) };
    unsafe { luaL_openlibs(lua) };
    unsafe { lua_pushlightuserdata(lua, state.cast()) };
    unsafe { lua_setfield(lua, LUA_REGISTRYINDEX, c"archPtr".as_ptr()) };
    unsafe { (*state).lua = lua };
    load_env(lua);
    let mut reader_data = ReadData { data: LUA_SANDBOX, i: 0 };
    if unsafe { lua_load(lua, Some(one_shot_read), (&raw mut reader_data).cast(), c"=machine.lua".as_ptr(), c"t".as_ptr()) } != LUA_OK {
        unsafe { lua_close(lua) };
        unsafe { nn_dealloc(alloc, state.cast(), size_of::<State>()) };
        return null_mut();
    }
    state.cast()
}
unsafe extern "C" fn teardown(computer: *mut nn_computer, state: *mut c_void, _userdata: *mut c_void) {
    let alloc = unsafe { nn_getAllocator(nn_getUniverse(computer)) };
    let state: *mut State = state.cast();
    unsafe { lua_close((*state).lua) };
    unsafe { nn_dealloc(alloc, state.cast(), size_of::<State>()) };
}
unsafe extern "C" fn get_memory_usage(_computer: *mut nn_computer, state: *mut c_void, _userdata: *mut c_void) -> usize {
    let state: *mut State = state.cast();
    unsafe { (*state).mem_usage }
}
unsafe extern "C" fn tick(computer: *mut nn_computer, state: *mut c_void, _userdata: *mut c_void) {
    let state: *mut State = state.cast();

    let res = unsafe { lua_resume((*state).lua, null_mut(), 0) };
    match res {
        LUA_OK => {
            // machine halted
            unsafe { nn_setCError(computer, c"machine halted".as_ptr()) };
        },
        LUA_YIELD => {},
        _ => {
            let s = unsafe { lua_tolstring((*state).lua, -1, null_mut()) };
            unsafe { nn_setError(computer, s) };
        }
    }
}
