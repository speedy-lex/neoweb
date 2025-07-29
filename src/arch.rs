use std::{ffi::c_void, ptr::null_mut};

use lua53_sys::{luaL_checklstring, luaL_error, luaL_loadbufferx, luaL_openlibs, lua_State, lua_close, lua_createtable, lua_getfield, lua_gettop, lua_isinteger, lua_isnumber, lua_newstate, lua_pushboolean, lua_pushcclosure, lua_pushinteger, lua_pushlightuserdata, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring, lua_resume, lua_setfield, lua_setglobal, lua_seti, lua_settable, lua_settop, lua_toboolean, lua_tointegerx, lua_tolstring, lua_tonumberx, lua_touserdata, lua_type, LUA_OK, LUA_REGISTRYINDEX, LUA_TBOOLEAN, LUA_TNUMBER, LUA_TSTRING, LUA_YIELD};
use neonucleus::ffi::{nn_Alloc, nn_addArgument, nn_alloc, nn_architecture, nn_clearError, nn_computer, nn_dealloc, nn_findComponent, nn_getAllocator, nn_getComponentAddress, nn_getComponentTable, nn_getComponentType, nn_getComputerMemoryTotal, nn_getError, nn_getReturn, nn_getReturnCount, nn_getState, nn_getUniverse, nn_getUptime, nn_invokeComponentMethod, nn_isOverheating, nn_isOverworked, nn_iterComponent, nn_resetCall, nn_resize, nn_setCError, nn_setError, nn_value, nn_values_boolean, nn_values_getType, nn_values_integer, nn_values_nil, nn_values_number, nn_values_string, NN_VALUE_ARRAY, NN_VALUE_BOOL, NN_VALUE_CSTR, NN_VALUE_INT, NN_VALUE_NIL, NN_VALUE_NUMBER, NN_VALUE_STR, NN_VALUE_TABLE};

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
unsafe fn get_alloc(lua: *mut lua_State) -> *mut nn_Alloc {
    unsafe { lua_getfield(lua, LUA_REGISTRYINDEX, c"archPtr".as_ptr()) };
    let state: *mut State = unsafe { lua_touserdata(lua, -1).cast() };
    unsafe { lua_pop(lua, 1) };
    unsafe { nn_getAllocator(nn_getUniverse((*state).computer)) }
}
unsafe fn get_value(lua: *mut lua_State, idx: i32) -> nn_value {
    let ty = unsafe { lua_type(lua, idx) };
    let alloc = unsafe { get_alloc(lua) };

    if ty == LUA_TBOOLEAN {
        unsafe { nn_values_boolean(lua_toboolean(lua, idx) != 0) }
    } else if unsafe { lua_type(lua, idx) } <= 0 {
        unsafe { nn_values_nil() }
    } else if ty == LUA_TSTRING {
        let mut len = 0;
        let s = unsafe { lua_tolstring(lua, idx, &raw mut len) };
        unsafe { nn_values_string(alloc, s, len) }
    } else if ty == LUA_TNUMBER && unsafe { lua_isinteger(lua, idx) } != 0 {
        unsafe { nn_values_integer(lua_tointegerx(lua, idx, null_mut()) as isize) }
    } else if ty == LUA_TNUMBER && unsafe { lua_isnumber(lua, idx) } != 0 {
        unsafe { nn_values_number(lua_tonumberx(lua, idx, null_mut())) }
    } else {
        unsafe { nn_values_nil() }
    }
}
unsafe fn push_value(lua: *mut lua_State, val: nn_value) {
    let t = unsafe { nn_values_getType(val) } as u32;
    match t {
        NN_VALUE_NIL => {
            unsafe { lua_pushnil(lua) };
        },
        NN_VALUE_INT => {
            unsafe { lua_pushinteger(lua, val.__bindgen_anon_1.integer as i64) };
        },
        NN_VALUE_NUMBER => {
            unsafe { lua_pushnumber(lua, val.__bindgen_anon_1.number) };
        },
        NN_VALUE_BOOL => {
            unsafe { lua_pushboolean(lua, val.__bindgen_anon_1.boolean as i32) };
        },
        NN_VALUE_STR => {
            unsafe { lua_pushlstring(lua, (*val.__bindgen_anon_1.string).data, (*val.__bindgen_anon_1.string).len) };
        },
        NN_VALUE_CSTR => {
            unsafe { lua_pushstring(lua, val.__bindgen_anon_1.cstring) };
        }
        NN_VALUE_ARRAY => {
            let arr = unsafe { val.__bindgen_anon_1.array };
            let len = unsafe { *arr }.len;
            unsafe { lua_createtable(lua, len as i32, 0) };
            let lua_val = unsafe { lua_gettop(lua) };
            for i in 0..len {
                unsafe { push_value(lua, *(*arr).values.add(i)) };
                unsafe { lua_seti(lua, lua_val, (i + 1) as i64) };
            }
        }
        NN_VALUE_TABLE => {
            let tbl = unsafe { val.__bindgen_anon_1.table };
            let len = unsafe { *tbl }.len;
            unsafe { lua_createtable(lua, 0, len as i32) };
            let lua_val = unsafe { lua_gettop(lua) };
            for i in 0..len {
                unsafe { push_value(lua, (*(*tbl).pairs.add(i)).key) };
                unsafe { push_value(lua, (*(*tbl).pairs.add(i)).val) };
                unsafe { lua_settable(lua, lua_val) };
            }
        },
        _ => {
            unsafe { luaL_error(lua, c"invalid return type".as_ptr()) };
        },
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
unsafe extern "C" fn computer_is_overheating(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };
    unsafe { lua_pushboolean(lua, nn_isOverheating((*state).computer) as i32) };
    1
}
unsafe extern "C" fn computer_is_overworked(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };
    unsafe { lua_pushboolean(lua, nn_isOverworked((*state).computer) as i32) };
    1
}
unsafe extern "C" fn computer_get_state(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };
    unsafe { lua_pushinteger(lua, nn_getState((*state).computer) as i64) };
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
unsafe extern "C" fn component_invoke(lua: *mut lua_State) -> i32 {
    let state = unsafe {
        get_state(lua)
    };

    let addr = unsafe { luaL_checklstring(lua, 1, null_mut()) };
    let method = unsafe { luaL_checklstring(lua, 2, null_mut()) };
    let argc = unsafe { lua_gettop(lua) } - 2;
    let component = unsafe { nn_findComponent((*state).computer, addr.cast_mut()) };
    if component.is_null() {
        unsafe { lua_pushnil(lua) };
        unsafe { lua_pushstring(lua, c"no such component".as_ptr()) };
        return 2
    }
    unsafe { nn_resetCall((*state).computer) };
    for i in 0..argc {
        unsafe { nn_addArgument((*state).computer, get_value(lua, i + 3)) };
    }
    if !unsafe { nn_invokeComponentMethod(component, method) } {
        unsafe { nn_resetCall((*state).computer) };
        unsafe { lua_pushnil(lua) };
        unsafe { lua_pushstring(lua, c"no such method".as_ptr()) };
        return 2;
    }
    if !unsafe { nn_getError((*state).computer).is_null() } {
        unsafe { nn_resetCall((*state).computer) };
        unsafe { luaL_error(lua, c"%s".as_ptr(), nn_getError((*state).computer)) };
    }
    let retc = unsafe { nn_getReturnCount((*state).computer) };
    for i in 0..retc {
        unsafe { push_value(lua, nn_getReturn((*state).computer, i)) };
    }
    unsafe { nn_resetCall((*state).computer) };
    retc as i32
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
    unsafe { lua_pushcclosure(lua, Some(computer_is_overworked), 0) };
    unsafe { lua_setfield(lua, computer, c"isOverworked".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_is_overheating), 0) };
    unsafe { lua_setfield(lua, computer, c"isOverheating".as_ptr()) };
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
    unsafe { lua_pushcclosure(lua, Some(computer_get_state), 0) };
    unsafe { lua_setfield(lua, computer, c"getState".as_ptr()) };
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
    unsafe { lua_pushcclosure(lua, Some(component_invoke), 0) };
    unsafe { lua_setfield(lua, component, c"invoke".as_ptr()) };
    // lua_pushcfunction(L, testLuaArch_component_slot);
    // lua_setfield(L, component, "slot");
    // lua_pushcfunction(L, testLuaArch_component_type);
    // lua_setfield(L, component, "type");
    unsafe { lua_setglobal(lua, c"component".as_ptr()) };

    unsafe { lua_createtable(lua, 0, 7) };
    let states = unsafe { lua_gettop(lua) };
    // lua_pushinteger(L, NN_STATE_SETUP);
    // lua_setfield(L, states, "setup");
    // lua_pushinteger(L, NN_STATE_RUNNING);
    // lua_setfield(L, states, "running");
    // lua_pushinteger(L, NN_STATE_BUSY);
    // lua_setfield(L, states, "busy");
    // lua_pushinteger(L, NN_STATE_BLACKOUT);
    // lua_setfield(L, states, "blackout");
    // lua_pushinteger(L, NN_STATE_CLOSING);
    // lua_setfield(L, states, "closing");
    // lua_pushinteger(L, NN_STATE_REPEAT);
    // lua_setfield(L, states, "REPEAT");
    // lua_pushinteger(L, NN_STATE_SWITCH);
    // lua_setfield(L, states, "switch");
    unsafe { lua_setglobal(lua, c"states".as_ptr()) };
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
