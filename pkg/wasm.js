function wasmSetCell(id, x, y, val) {
    const t = String.fromCodePoint(val);
    setCell(id, x, y, t);
}

function tickComputer() {
    try {
        wasm.tick()
    } catch(e) {
        if (e instanceof WebAssembly.RuntimeError && e.message.includes("unreachable")) {
            alert("panic: " + e.message);
        } else {
            alert(e);
        }
    }
    requestAnimationFrame(tickComputer);
}

function runComputer() {
    try {
        wasm.init();
    } catch(e) {
        if (e instanceof WebAssembly.RuntimeError && e.message.includes("unreachable")) {
            alert("panic: " + e.message);
        } else {
            alert(e);
        }
    }
    requestAnimationFrame(tickComputer);
}

const importObject = {
    neoweb_console: {
        set_cell: wasmSetCell,
    },
    neoweb_utils: {
        get_time: () => {
            Date.now() / 1000
        }
    },
    libc: {
        get_unix_time_s: () => {
            Math.floor(Date.now() / 1000)
        }
    }
};
let wasm = undefined;
createScreen(document.getElementById("container"), 80, 25);

const response = WebAssembly.instantiateStreaming(fetch("neoweb.wasm"), importObject).then(
    (obj) => {wasm = obj.instance.exports; runComputer()},
);