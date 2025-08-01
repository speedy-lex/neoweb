function wasmSetCell(id, x, y, val) {
    const t = String.fromCodePoint(val);
    setCell(id, x, y, t);
}

async function fetchFileBytes(url) {
    const response = await fetch(url);

    if (!response.ok) {
        throw new Error(`Failed to fetch file: ${response.status}`);
    }

    const arrayBuffer = await response.arrayBuffer();
    const byteArray = new Uint8Array(arrayBuffer);

    return byteArray;
}

function tickComputer() {
    try {
        wasm.tick()
        requestAnimationFrame(tickComputer);
    } catch(e) {
        if (e instanceof WebAssembly.RuntimeError && e.message.includes("unreachable")) {
            console.error("panic: " + e.message);
        } else {
            console.dir(e);
        }
    }
}

const ch_to_oc_map = {}
ch_to_oc_map["a"] = 0x1E;
ch_to_oc_map["b"] = 0x30;
ch_to_oc_map["c"] = 0x2E;
ch_to_oc_map["d"] = 0x20;
ch_to_oc_map["e"] = 0x12;
ch_to_oc_map["f"] = 0x21;
ch_to_oc_map["g"] = 0x22;
ch_to_oc_map["h"] = 0x23;
ch_to_oc_map["i"] = 0x17;
ch_to_oc_map["j"] = 0x24;
ch_to_oc_map["k"] = 0x25;
ch_to_oc_map["l"] = 0x26;
ch_to_oc_map["m"] = 0x32;
ch_to_oc_map["n"] = 0x31;
ch_to_oc_map["o"] = 0x18;
ch_to_oc_map["p"] = 0x19;
ch_to_oc_map["q"] = 0x10;
ch_to_oc_map["r"] = 0x13;
ch_to_oc_map["s"] = 0x1F;
ch_to_oc_map["t"] = 0x14;
ch_to_oc_map["u"] = 0x16;
ch_to_oc_map["v"] = 0x2F;
ch_to_oc_map["w"] = 0x11;
ch_to_oc_map["x"] = 0x2D;
ch_to_oc_map["y"] = 0x15;
ch_to_oc_map["z"] = 0x2C;
ch_to_oc_map["Enter"] = 0x1C;
ch_to_oc_map[" "] = 0x39;
ch_to_oc_map["\'"] = 0x28;
ch_to_oc_map[","] = 0x33;
ch_to_oc_map["-"] = 0x0C;
ch_to_oc_map["."] = 0x34;
ch_to_oc_map["/"] = 0x35;
ch_to_oc_map["0"] = 0x0B;
ch_to_oc_map["1"] = 0x02;
ch_to_oc_map["2"] = 0x03;
ch_to_oc_map["3"] = 0x04;
ch_to_oc_map["4"] = 0x05;
ch_to_oc_map["5"] = 0x06;
ch_to_oc_map["6"] = 0x07;
ch_to_oc_map["7"] = 0x08;
ch_to_oc_map["8"] = 0x09;
ch_to_oc_map["9"] = 0x0A;
ch_to_oc_map[";"] = 0x27;
ch_to_oc_map["="] = 0x0D;
ch_to_oc_map["["] = 0x1A;
ch_to_oc_map["\\"] = 0x2B;
ch_to_oc_map["]"] = 0x1B;
ch_to_oc_map["`"] = 0x29;
ch_to_oc_map["Backspace"] = 0x0E;
ch_to_oc_map["Shift"] = 0x2A;
ch_to_oc_map["Control"] = 0x1D;

document.onkeydown = function(e) {
    e.preventDefault();
    let key = e.key;
    if (e.key == "Enter") { key = "\r" }
    if (e.key == "Backspace") { key = "\b" }
    if (e.key == "Shift") { key = String.fromCodePoint(0) }
    if (e.key == "Control") { key = String.fromCodePoint(0) }
    if (e.key == "Tab") { key = "\t" }
    if (e.key == "Meta") { return }
    wasm.on_key(key.charCodeAt(0), ch_to_oc_map[e.key] || 0, e.type == "keyup");
}
document.onkeyup = document.onkeydown

async function runComputer() {
    try {
        wasm.init();
        let eeprom = await fetchFileBytes("luaBios.lua");
        let code = wasm.alloc_block(eeprom.byteLength);
        let data = wasm.alloc_block(1024);
        const wasmMemory = new Uint8Array(wasm.memory.buffer, code, eeprom.byteLength);
        wasmMemory.set(eeprom);
        wasm.load_eeprom(code, eeprom.byteLength, eeprom.byteLength, data, 1024, 0);

        let openos = await fetchFileBytes("openos.ntar");
        let alloc = wasm.alloc_block(openos.byteLength);
        const wasmOpenos = new Uint8Array(wasm.memory.buffer, alloc, openos.byteLength);
        wasmOpenos.set(openos);
        wasm.load_vfs(alloc, openos.byteLength);

        requestAnimationFrame(tickComputer);
    } catch(e) {
        if (e instanceof WebAssembly.RuntimeError && e.message.includes("unreachable")) {
            console.error("panic: " + e.message);
        } else {
            console.dir(e);
        }
        return;
    }
}

const importObject = {
    neoweb_console: {
        set_cell: wasmSetCell,
    },
    neoweb_utils: {
        get_time: () => {
            return Date.now() / 1000;
        },
        debug_log: (ptr) => {
            const wasmMemory = new Uint8Array(wasm.memory.buffer);
            let end = ptr;
            while (wasmMemory[end] !== 0) end++;
            const slice = wasmMemory.subarray(ptr, end);
            const str = new TextDecoder("utf-8").decode(slice);
            console.log('[wasm]:', str);
        },
        debug_error: (ptr) => {
            const wasmMemory = new Uint8Array(wasm.memory.buffer);
            let end = ptr;
            while (wasmMemory[end] !== 0) end++;
            const slice = wasmMemory.subarray(ptr, end);
            const str = new TextDecoder("utf-8").decode(slice);
            console.error('[wasm]:', str);
        }
    },
    libc: {
        get_unix_time_s: () => {
            return BigInt(Math.floor(Date.now() / 1000));
        }
    },
    env: {
        __ubsan_handle_type_mismatch_v1: (x, y) => {},
        __ubsan_handle_pointer_overflow: (x, y, z) => {},
        __ubsan_handle_divrem_overflow: (x, y, z) => {},
        __ubsan_handle_add_overflow: (x, y, z) => {},
        __ubsan_handle_float_cast_overflow: (x, y) => {},
        __ubsan_handle_out_of_bounds: (x, y) => {},
        __ubsan_handle_vla_bound_not_positive: (x, y) => {},
        __ubsan_handle_mul_overflow: (x, y, z) => {},
        __ubsan_handle_sub_overflow: (x, y, z) => {},
        __ubsan_handle_shift_out_of_bounds: (x, y, z) => {},
    }
};
let wasm = undefined;
createScreen(document.getElementById("container"), 80, 25);

const response = WebAssembly.instantiateStreaming(fetch("neoweb.wasm"), importObject).then(
    async (obj) => {wasm = obj.instance.exports; await runComputer()},
);