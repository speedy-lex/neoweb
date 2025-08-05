use std::{
    ffi::{c_char, c_void},
    ptr::{null, null_mut},
};

use lua53_sys::{
    LUA_OK, LUA_REGISTRYINDEX, LUA_TBOOLEAN, LUA_TNUMBER, LUA_TSTRING, LUA_YIELD, lua_State,
    lua_checkstack, lua_close, lua_createtable, lua_getfield, lua_gettop, lua_isinteger,
    lua_isnumber, lua_newstate, lua_pushboolean, lua_pushcclosure, lua_pushinteger,
    lua_pushlightuserdata, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring,
    lua_resume, lua_setfield, lua_setglobal, lua_seti, lua_settable, lua_settop, lua_toboolean,
    lua_tointegerx, lua_tolstring, lua_tonumberx, lua_touserdata, lua_type, luaL_argerror,
    luaL_checkinteger, luaL_checklstring, luaL_checknumber, luaL_error, luaL_loadbufferx,
    luaL_openlibs,
};
use neonucleus::ffi::{
    NN_MAX_ARGS, NN_STATE_BLACKOUT, NN_STATE_BUSY, NN_STATE_CLOSING, NN_STATE_REPEAT,
    NN_STATE_RUNNING, NN_STATE_SETUP, NN_STATE_SWITCH, NN_VALUE_ARRAY, NN_VALUE_BOOL,
    NN_VALUE_CSTR, NN_VALUE_INT, NN_VALUE_NIL, NN_VALUE_NUMBER, NN_VALUE_STR, NN_VALUE_TABLE,
    nn_Alloc, nn_addArgument, nn_addHeat, nn_alloc, nn_architecture, nn_clearError, nn_computer,
    nn_dealloc, nn_deallocStr, nn_fetchSignalValue, nn_findComponent, nn_getAllocator,
    nn_getArchitecture, nn_getComponentAddress, nn_getComponentSlot, nn_getComponentTable,
    nn_getComponentType, nn_getComputerAddress, nn_getComputerMemoryTotal, nn_getEnergy,
    nn_getError, nn_getMaxEnergy, nn_getReturn, nn_getReturnCount, nn_getState,
    nn_getSupportedArchitecture, nn_getTableMethod, nn_getTemperature, nn_getTmpAddress,
    nn_getUniverse, nn_getUptime, nn_indexUser, nn_invokeComponentMethod, nn_isMethodEnabled,
    nn_isOverheating, nn_isOverworked, nn_iterComponent, nn_methodDoc, nn_popSignal, nn_pushSignal,
    nn_resetCall, nn_resize, nn_setCError, nn_setError, nn_setNextArchitecture, nn_setState,
    nn_signalSize, nn_strcmp, nn_unicode_char, nn_unicode_indexPermissive,
    nn_unicode_lenPermissive, nn_value, nn_values_boolean, nn_values_dropAll, nn_values_getType,
    nn_values_integer, nn_values_nil, nn_values_number, nn_values_string,
};

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

unsafe extern "C" fn lua_alloc(
    state: *mut c_void,
    ptr: *mut c_void,
    old: usize,
    new: usize,
) -> *mut c_void {
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
        if state.mem_usage - actual_old + new > unsafe { nn_getComputerMemoryTotal(state.computer) }
        {
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
        unsafe { nn_values_integer(lua_tointegerx(lua, idx, null_mut())) }
    } else if ty == LUA_TNUMBER && unsafe { lua_isnumber(lua, idx) } != 0 {
        unsafe { nn_values_number(lua_tonumberx(lua, idx, null_mut())) }
    } else {
        unsafe { nn_values_nil() }
    }
}
unsafe fn push_value(lua: *mut lua_State, val: nn_value) {
    let t = unsafe { nn_values_getType(val) } as i32;
    match t {
        NN_VALUE_NIL => {
            unsafe { lua_pushnil(lua) };
        }
        NN_VALUE_INT => {
            unsafe { lua_pushinteger(lua, val.__bindgen_anon_1.integer) };
        }
        NN_VALUE_NUMBER => {
            unsafe { lua_pushnumber(lua, val.__bindgen_anon_1.number) };
        }
        NN_VALUE_BOOL => {
            unsafe { lua_pushboolean(lua, val.__bindgen_anon_1.boolean as i32) };
        }
        NN_VALUE_STR => {
            unsafe {
                lua_pushlstring(
                    lua,
                    (*val.__bindgen_anon_1.string).data,
                    (*val.__bindgen_anon_1.string).len,
                )
            };
        }
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
        }
        _ => {
            unsafe { luaL_error(lua, c"invalid return type".as_ptr()) };
        }
    }
}

unsafe fn pushlstring_safe(lua: *mut lua_State, s: *const i8, len: usize) -> *const c_char {
    if (unsafe { lua_checkstack(lua, 1) } == 0) {
        return null();
    }
    let state = unsafe { get_state(lua) };
    let free_space =
        unsafe { nn_getComputerMemoryTotal((*state).computer) } - unsafe { (*state).mem_usage };
    if (len * 2 + 64) > free_space {
        // dk how much space this really needs and its unstable so :/
        return null();
    }
    unsafe { lua_pushlstring(lua, s, len) }
}

unsafe fn lua_pop(lua: *mut lua_State, n: i32) {
    unsafe { lua_settop(lua, -(n) - 1) }
}
unsafe fn get_state(lua: *mut lua_State) -> *mut State {
    unsafe { lua_getfield(lua, LUA_REGISTRYINDEX, c"archPtr".as_ptr()) };
    let state = unsafe { lua_touserdata(lua, -1) };
    unsafe { lua_pop(lua, 1) };
    state.cast()
}
unsafe extern "C" fn computer_clear_error(lua: *mut lua_State) -> i32 {
    let state = unsafe { get_state(lua) };
    unsafe { nn_clearError((*state).computer) };
    0
}
unsafe extern "C" fn computer_uptime(lua: *mut lua_State) -> i32 {
    let state = unsafe { get_state(lua) };
    unsafe { lua_pushnumber(lua, nn_getUptime((*state).computer)) };
    1
}
unsafe extern "C" fn computer_is_overheating(lua: *mut lua_State) -> i32 {
    let state = unsafe { get_state(lua) };
    unsafe { lua_pushboolean(lua, nn_isOverheating((*state).computer) as i32) };
    1
}
unsafe extern "C" fn computer_is_overworked(lua: *mut lua_State) -> i32 {
    let state = unsafe { get_state(lua) };
    unsafe { lua_pushboolean(lua, nn_isOverworked((*state).computer) as i32) };
    1
}
unsafe extern "C" fn computer_get_state(lua: *mut lua_State) -> i32 {
    let state = unsafe { get_state(lua) };
    unsafe { lua_pushinteger(lua, nn_getState((*state).computer) as i64) };
    1
}
unsafe extern "C" fn computer_beep(_lua: *mut lua_State) -> i32 {
    // TODO: beep
    0
}
unsafe extern "C" fn component_list(lua: *mut lua_State) -> i32 {
    let state = unsafe { get_state(lua) };
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
    let state = unsafe { get_state(lua) };

    let addr = unsafe { luaL_checklstring(lua, 1, null_mut()) };
    let method = unsafe { luaL_checklstring(lua, 2, null_mut()) };
    let argc = unsafe { lua_gettop(lua) } - 2;
    let component = unsafe { nn_findComponent((*state).computer, addr.cast_mut()) };
    if component.is_null() {
        unsafe { lua_pushnil(lua) };
        unsafe { lua_pushstring(lua, c"no such component".as_ptr()) };
        return 2;
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

unsafe extern "C" fn computer_used_memory(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushinteger(lua, (*state).mem_usage as i64);
        1
    }
}

unsafe extern "C" fn computer_free_memory(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let total = nn_getComputerMemoryTotal((*state).computer);
        lua_pushinteger(lua, (total - (*state).mem_usage) as i64);
        1
    }
}

unsafe extern "C" fn computer_total_memory(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushinteger(lua, nn_getComputerMemoryTotal((*state).computer) as i64);
        1
    }
}

unsafe extern "C" fn computer_energy(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushinteger(lua, nn_getEnergy((*state).computer) as i64);
        1
    }
}

unsafe extern "C" fn computer_max_energy(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushinteger(lua, nn_getMaxEnergy((*state).computer) as i64);
        1
    }
}

unsafe extern "C" fn computer_get_architecture(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushstring(lua, (*nn_getArchitecture((*state).computer)).archName);
        1
    }
}

unsafe extern "C" fn computer_get_architectures(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_createtable(lua, 3, 0);
        let arr = lua_gettop(lua);
        let mut i = 0;
        loop {
            let arch = nn_getSupportedArchitecture((*state).computer, i);
            if arch.is_null() {
                break;
            }
            i += 1;
            lua_pushstring(lua, (*arch).archName);
            lua_seti(lua, arr, i as i64);
        }
        1
    }
}

unsafe extern "C" fn computer_set_architecture(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let requested = luaL_checklstring(lua, 1, null_mut());
        loop {
            let arch = nn_getArchitecture((*state).computer);
            if arch.is_null() {
                break;
            }
            if nn_strcmp((*arch).archName, requested) == 0 {
                nn_setState((*state).computer, NN_STATE_SWITCH);
                nn_setNextArchitecture((*state).computer, arch);
                return 0;
            }
        }
        luaL_error(lua, c"unsupported architecture: %s".as_ptr(), requested);
        0
    }
}

unsafe extern "C" fn computer_get_temperature(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushnumber(lua, nn_getTemperature((*state).computer));
        1
    }
}

unsafe extern "C" fn computer_add_heat(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        nn_addHeat((*state).computer, luaL_checknumber(lua, 1));
        0
    }
}

unsafe extern "C" fn computer_address(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushstring(lua, nn_getComputerAddress((*state).computer));
        1
    }
}

unsafe extern "C" fn computer_tmp_address(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        lua_pushstring(lua, nn_getTmpAddress((*state).computer));
        1
    }
}

unsafe extern "C" fn computer_users(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let mut i = 0;
        loop {
            let name = nn_indexUser((*state).computer, i);
            if name.is_null() {
                break;
            }
            lua_pushstring(lua, name);
            i += 1;
        }
        i as i32
    }
}

unsafe extern "C" fn computer_set_state(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let s = luaL_checkinteger(lua, 1);
        nn_setState((*state).computer, s as i32);
        1
    }
}

unsafe extern "C" fn computer_push_signal(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        luaL_checklstring(lua, 1, null_mut());
        let argc = lua_gettop(lua);
        if argc > NN_MAX_ARGS {
            luaL_error(lua, c"too many arguments".as_ptr());
        }
        let mut args: Vec<nn_value> = Vec::with_capacity(argc as usize);
        for i in 0..argc {
            args.push(get_value(lua, i + 1));
        }
        let err = nn_pushSignal((*state).computer, args.as_mut_ptr(), argc as usize);
        if !err.is_null() {
            nn_values_dropAll(args.as_mut_ptr(), argc as usize);
            luaL_error(lua, c"%s".as_ptr(), err);
            return 0;
        }
        0
    }
}

unsafe extern "C" fn computer_pop_signal(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let retc = nn_signalSize((*state).computer);
        for i in 0..retc {
            push_value(lua, nn_fetchSignalValue((*state).computer, i));
        }
        nn_popSignal((*state).computer);
        retc as i32
    }
}

unsafe extern "C" fn component_doc(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let addr = luaL_checklstring(lua, 1, null_mut());
        let method = luaL_checklstring(lua, 2, null_mut());
        let component = nn_findComponent((*state).computer, addr as *mut _);
        if component.is_null() {
            lua_pushnil(lua);
            lua_pushstring(lua, c"no such component".as_ptr());
            return 2;
        }
        let doc = nn_methodDoc(nn_getComponentTable(component), method);
        if doc.is_null() {
            lua_pushnil(lua);
        } else {
            lua_pushstring(lua, doc);
        }
        1
    }
}

unsafe extern "C" fn component_fields(lua: *mut lua_State) -> i32 {
    unsafe {
        lua_createtable(lua, 0, 0);
        1
    }
}

unsafe extern "C" fn component_slot(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let addr = luaL_checklstring(lua, 1, null_mut());
        let component = nn_findComponent((*state).computer, addr as *mut _);
        if component.is_null() {
            lua_pushnil(lua);
            lua_pushstring(lua, c"no such component".as_ptr());
            return 2;
        }

        lua_pushinteger(lua, nn_getComponentSlot(component) as i64);
        1
    }
}

unsafe extern "C" fn component_type(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let addr = luaL_checklstring(lua, 1, null_mut());
        let component = nn_findComponent((*state).computer, addr as *mut _);
        if component.is_null() {
            lua_pushnil(lua);
            lua_pushstring(lua, c"no such component".as_ptr());
            return 2;
        }

        lua_pushstring(lua, nn_getComponentType(nn_getComponentTable(component)));
        1
    }
}

unsafe extern "C" fn component_methods(lua: *mut lua_State) -> i32 {
    unsafe {
        let state = get_state(lua);
        let addr = luaL_checklstring(lua, 1, null_mut());
        let component = nn_findComponent((*state).computer, addr as *mut _);
        if component.is_null() {
            lua_pushnil(lua);
            lua_pushstring(lua, c"no such component".as_ptr());
            return 2;
        }
        let table = nn_getComponentTable(component);
        lua_createtable(lua, 0, 0);
        let methods = lua_gettop(lua);

        let mut i = 0;
        loop {
            let mut direct = false;
            let name = nn_getTableMethod(table, i, &mut direct as *mut _);
            if name.is_null() {
                break;
            }
            i += 1;
            if !nn_isMethodEnabled(component, name) {
                continue;
            }
            lua_pushboolean(lua, direct as i32);
            lua_setfield(lua, methods, name);
        }
        1
    }
}

unsafe extern "C" fn unicode_len(lua: *mut lua_State) -> i32 {
    let str = unsafe { luaL_checklstring(lua, 1, null_mut()) };
    unsafe { lua_pushinteger(lua, nn_unicode_lenPermissive(str) as i64) };
    1
}

unsafe extern "C" fn unicode_wlen(lua: *mut lua_State) -> i32 {
    let str = unsafe { luaL_checklstring(lua, 1, null_mut()) };
    unsafe { lua_pushinteger(lua, nn_unicode_lenPermissive(str) as i64) };
    1
}

unsafe extern "C" fn unicode_char(lua: *mut lua_State) -> i32 {
    let argc = unsafe { lua_gettop(lua) };
    let mut codepoints: Vec<u32> = Vec::with_capacity(argc as usize);
    for i in 0..argc {
        if unsafe { lua_isinteger(lua, i + 1) } == 0 {
            unsafe { luaL_argerror(lua, i + 1, c"integer expected".as_ptr()) };
            return 0;
        }
        codepoints.push(unsafe { lua_tointegerx(lua, i + 1, null_mut()) } as u32);
    }
    let alloc = unsafe { get_alloc(lua) };
    let str = unsafe { nn_unicode_char(alloc, codepoints.as_mut_ptr(), codepoints.len()) };
    let res = unsafe { lua_pushstring(lua, str) };
    unsafe { nn_deallocStr(alloc, str) };
    if res.is_null() {
        unsafe { luaL_error(lua, c"out of memory".as_ptr()) };
    }
    1
}
unsafe extern "C" fn unicode_sub(lua: *mut lua_State) -> i32 {
    let str = unsafe { luaL_checklstring(lua, 1, null_mut()) };
    let mut start = unsafe { luaL_checkinteger(lua, 2) };
    let len = unsafe { nn_unicode_lenPermissive(str) };
    let mut stop = len as i64;
    if unsafe { lua_isinteger(lua, 3) } != 0 {
        stop = unsafe { luaL_checkinteger(lua, 3) };
    }
    // OpenOS does this...
    if len == 0 {
        unsafe { lua_pushstring(lua, c"".as_ptr()) };
        return 1;
    }

    if start == 0 {
        start = 1;
    }
    if stop == 0 {
        unsafe { lua_pushstring(lua, c"".as_ptr()) };
        return 1;
    }
    if start < 0 {
        start = len as i64 + start + 1;
    }
    if stop < 0 {
        stop = len as i64 + stop + 1;
    }

    if stop > len as i64 {
        stop = len as i64;
    }

    if start > stop {
        unsafe { lua_pushstring(lua, c"".as_ptr()) };
        return 1;
    }

    let start_byte = unsafe { nn_unicode_indexPermissive(str, (start - 1) as usize) };
    let term_byte = unsafe { nn_unicode_indexPermissive(str, stop as usize) };

    let res = unsafe {
        pushlstring_safe(
            lua,
            str.byte_offset(start_byte),
            (term_byte - start_byte) as usize,
        )
    };
    if res.is_null() { // thanks Calion
        unsafe { luaL_error(lua, c"out of memory".as_ptr()) };
    }
    1
}

fn load_env(lua: *mut lua_State) {
    unsafe { lua_createtable(lua, 0, 10) };
    let computer = unsafe { lua_gettop(lua) };
    unsafe { lua_pushcclosure(lua, Some(computer_clear_error), 0) };
    unsafe { lua_setfield(lua, computer, c"clearError".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_used_memory), 0) };
    unsafe { lua_setfield(lua, computer, c"usedMemory".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_free_memory), 0) };
    unsafe { lua_setfield(lua, computer, c"freeMemory".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_total_memory), 0) };
    unsafe { lua_setfield(lua, computer, c"totalMemory".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_address), 0) };
    unsafe { lua_setfield(lua, computer, c"address".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_tmp_address), 0) };
    unsafe { lua_setfield(lua, computer, c"tmpAddress".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_uptime), 0) };
    unsafe { lua_setfield(lua, computer, c"uptime".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_beep), 0) };
    unsafe { lua_setfield(lua, computer, c"beep".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_energy), 0) };
    unsafe { lua_setfield(lua, computer, c"energy".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_max_energy), 0) };
    unsafe { lua_setfield(lua, computer, c"maxEnergy".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_get_architecture), 0) };
    unsafe { lua_setfield(lua, computer, c"getArchitecture".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_get_architectures), 0) };
    unsafe { lua_setfield(lua, computer, c"getArchitectures".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_set_architecture), 0) };
    unsafe { lua_setfield(lua, computer, c"setArchitecture".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_is_overworked), 0) };
    unsafe { lua_setfield(lua, computer, c"isOverworked".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_is_overheating), 0) };
    unsafe { lua_setfield(lua, computer, c"isOverheating".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_get_temperature), 0) };
    unsafe { lua_setfield(lua, computer, c"getTemperature".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_add_heat), 0) };
    unsafe { lua_setfield(lua, computer, c"addHeat".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_push_signal), 0) };
    unsafe { lua_setfield(lua, computer, c"pushSignal".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_pop_signal), 0) };
    unsafe { lua_setfield(lua, computer, c"popSignal".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_users), 0) };
    unsafe { lua_setfield(lua, computer, c"users".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_get_state), 0) };
    unsafe { lua_setfield(lua, computer, c"getState".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(computer_set_state), 0) };
    unsafe { lua_setfield(lua, computer, c"setState".as_ptr()) };
    unsafe { lua_setglobal(lua, c"computer".as_ptr()) };

    unsafe { lua_createtable(lua, 0, 10) };
    let component = unsafe { lua_gettop(lua) };
    unsafe { lua_pushcclosure(lua, Some(component_list), 0) };
    unsafe { lua_setfield(lua, component, c"list".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(component_doc), 0) };
    unsafe { lua_setfield(lua, component, c"doc".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(component_fields), 0) };
    unsafe { lua_setfield(lua, component, c"fields".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(component_methods), 0) };
    unsafe { lua_setfield(lua, component, c"methods".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(component_invoke), 0) };
    unsafe { lua_setfield(lua, component, c"invoke".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(component_slot), 0) };
    unsafe { lua_setfield(lua, component, c"slot".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(component_type), 0) };
    unsafe { lua_setfield(lua, component, c"type".as_ptr()) };
    unsafe { lua_setglobal(lua, c"component".as_ptr()) };

    unsafe { lua_createtable(lua, 0, 7) };
    let states = unsafe { lua_gettop(lua) };
    unsafe { lua_pushinteger(lua, NN_STATE_SETUP as _) };
    unsafe { lua_setfield(lua, states, c"setup".as_ptr()) };
    unsafe { lua_pushinteger(lua, NN_STATE_RUNNING as _) };
    unsafe { lua_setfield(lua, states, c"running".as_ptr()) };
    unsafe { lua_pushinteger(lua, NN_STATE_BUSY as _) };
    unsafe { lua_setfield(lua, states, c"busy".as_ptr()) };
    unsafe { lua_pushinteger(lua, NN_STATE_BLACKOUT as _) };
    unsafe { lua_setfield(lua, states, c"blackout".as_ptr()) };
    unsafe { lua_pushinteger(lua, NN_STATE_CLOSING as _) };
    unsafe { lua_setfield(lua, states, c"closing".as_ptr()) };
    unsafe { lua_pushinteger(lua, NN_STATE_REPEAT as _) };
    unsafe { lua_setfield(lua, states, c"repeat".as_ptr()) };
    unsafe { lua_pushinteger(lua, NN_STATE_SWITCH as _) };
    unsafe { lua_setfield(lua, states, c"switch".as_ptr()) };
    unsafe { lua_setglobal(lua, c"states".as_ptr()) };

    unsafe { lua_createtable(lua, 0, 20) };
    let unicode = unsafe { lua_gettop(lua) };
    unsafe { lua_pushcclosure(lua, Some(unicode_sub), 0) };
    unsafe { lua_setfield(lua, unicode, c"sub".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(unicode_len), 0) };
    unsafe { lua_setfield(lua, unicode, c"len".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(unicode_wlen), 0) };
    unsafe { lua_setfield(lua, unicode, c"wlen".as_ptr()) };
    unsafe { lua_pushcclosure(lua, Some(unicode_char), 0) };
    unsafe { lua_setfield(lua, unicode, c"char".as_ptr()) };
    unsafe { lua_setglobal(lua, c"unicode".as_ptr()) };
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
    if unsafe {
        luaL_loadbufferx(
            lua,
            LUA_SANDBOX.as_ptr().cast(),
            LUA_SANDBOX.len(),
            c"=machine.lua".as_ptr(),
            c"t".as_ptr(),
        )
    } != LUA_OK
    {
        unsafe { lua_close(lua) };
        unsafe { nn_dealloc(alloc, state.cast(), size_of::<State>()) };
        return null_mut();
    }
    state.cast()
}
unsafe extern "C" fn teardown(
    computer: *mut nn_computer,
    state: *mut c_void,
    _userdata: *mut c_void,
) {
    let alloc = unsafe { nn_getAllocator(nn_getUniverse(computer)) };
    let state: *mut State = state.cast();
    unsafe { lua_close((*state).lua) };
    unsafe { nn_dealloc(alloc, state.cast(), size_of::<State>()) };
}
unsafe extern "C" fn get_memory_usage(
    _computer: *mut nn_computer,
    state: *mut c_void,
    _userdata: *mut c_void,
) -> usize {
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
        }
        LUA_YIELD => {}
        _ => {
            let s = unsafe { lua_tolstring((*state).lua, -1, null_mut()) };
            unsafe { nn_setError(computer, s) };
        }
    }
}
