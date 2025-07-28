function wasmSetCell(id, x, y, val) {
    const t = String.fromCodePoint(val);
    setCell(id, x, y, t);
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

function runComputer() {
    try {
        wasm.init();
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
    (obj) => {wasm = obj.instance.exports; runComputer()},
);