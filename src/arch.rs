use std::{ffi::c_void, ptr::null_mut};

use lua53_sys::{luaL_loadbufferx, luaL_openlibs, lua_State, lua_close, lua_createtable, lua_getfield, lua_gettop, lua_newstate, lua_pushcclosure, lua_pushlightuserdata, lua_pushnumber, lua_pushstring, lua_resume, lua_setfield, lua_setglobal, lua_settop, lua_tolstring, lua_touserdata, LUA_OK, LUA_REGISTRYINDEX, LUA_YIELD};
use neonucleus::ffi::{nn_alloc, nn_architecture, nn_clearError, nn_computer, nn_dealloc, nn_getAllocator, nn_getComponentAddress, nn_getComponentTable, nn_getComponentType, nn_getComputerMemoryTotal, nn_getUniverse, nn_getUptime, nn_iterComponent, nn_resize, nn_setCError, nn_setError};

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
unsafe extern "C" fn computer_uptime(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };
    unsafe { lua_pushnumber(lua, nn_getUptime((*state).computer)) };
    1
}
unsafe extern "C" fn component_list(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };
    unsafe { lua_createtable(lua, 0, 10) };
    let mut iter = 0;
    let list = unsafe { lua_gettop(lua) };
    loop {
        let component = unsafe { nn_iterComponent((*state).computer, &raw mut iter) };
        if component.is_null() {
            break;
        }
        let table = unsafe { nn_getComponentTable(component) };
        let addr = unsafe { nn_getComponentAddress(component) };
        let ty = unsafe { nn_getComponentType(table) };

        unsafe { lua_pushstring(lua, ty) };
        unsafe { lua_setfield(lua, list, addr) };
    }
    1
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
    unsafe { lua_pushcclosure(lua, Some(computer_uptime), 0) };
    unsafe { lua_setfield(lua, computer, c"uptime".as_ptr()) };
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

    unsafe { lua_createtable(lua, 0, 10) };
    let component = unsafe { lua_gettop(lua) };
    unsafe { lua_pushcclosure(lua, Some(component_list), 0) };
    unsafe { lua_setfield(lua, component, c"list".as_ptr()) };
    // lua_pushcfunction(L, testLuaArch_component_doc);
    // lua_setfield(L, component, "doc");
    // lua_pushcfunction(L, testLuaArch_component_fields);
    // lua_setfield(L, component, "fields");
    // lua_pushcfunction(L, testLuaArch_component_methods);
    // lua_setfield(L, component, "methods");
    // lua_pushcfunction(L, testLuaArch_component_invoke);
    // lua_setfield(L, component, "invoke");
    // lua_pushcfunction(L, testLuaArch_component_slot);
    // lua_setfield(L, component, "slot");
    // lua_pushcfunction(L, testLuaArch_component_type);
    // lua_setfield(L, component, "type");
    unsafe { lua_setglobal(lua, c"component".as_ptr()) };
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
    if unsafe { luaL_loadbufferx(lua, LUA_SANDBOX.as_ptr().cast(), LUA_SANDBOX.len(), c"=machine.lua".as_ptr(), c"t".as_ptr()) } != LUA_OK {
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
