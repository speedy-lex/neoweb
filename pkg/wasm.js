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