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
        let buffer = wasm.alloc_eeprom(eeprom.byteLength);
        const wasmMemory = new Uint8Array(wasm.memory.buffer, buffer, eeprom.byteLength);
        wasmMemory.set(eeprom);
        wasm.load_eeprom(buffer, eeprom.byteLength, eeprom.byteLength, 0, 0, 0);
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
        }
    },
    libc: {
        get_unix_time_s: () => {
            return BigInt(Math.floor(Date.now() / 1000));
        }
    }
};
let wasm = undefined;
createScreen(document.getElementById("container"), 80, 25);

const response = WebAssembly.instantiateStreaming(fetch("neoweb.wasm"), importObject).then(
    async (obj) => {wasm = obj.instance.exports; await runComputer()},
);