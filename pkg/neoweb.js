let screens = [];

async function fetchFileBytesCompressed(url) {
    const response = await fetch(url);

    if (!response.ok) {
        throw new Error(`Failed to fetch file: ${response.status}`);
    }

    const decompressedStream = response.body.pipeThrough(new DecompressionStream("gzip"));

    const arrayBuffer = await new Response(decompressedStream).arrayBuffer();
    const byteArray = new Uint8Array(arrayBuffer);

    return byteArray;
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

function createScreen(element, cols, rows) {
    let child = document.createElement("div");
    child.classList.add("screen");
    child.tabIndex = 0;
    child.style = "--cols:" + cols + ";--rows:" + rows + ";"
    for (let i = 0; i < cols * rows; i++) {
        const cell = document.createElement("span");
        cell.classList.add("cell");
        child.appendChild(cell);
    }
    element.appendChild(child);
    screens.push({
        element: child,
        cols: cols,
        rows: rows,
    });
    return screens.length - 1;
}

function getScreenElement(id) {
    return screens[id].element;
}

function setCell(id, x, y, val) {
    const i = x + 80 * y;
    screens[id].element.children[i].innerText = val
}

async function addDefaultComputer() {
    let computer = new window.nwComputer();
    new window.nwScreen(computer, document.getElementById('container'), 1);
    computer.add_eeprom(await fetchFileBytes('luaBios.lua'));
    computer.add_vfs(await fetchFileBytesCompressed('openos.ntar.gz'));
    computer.start_ticking();
}
