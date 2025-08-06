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
    const wrapper = document.createElement("div");
    wrapper.classList.add("screen-wrapper");
    wrapper.style = "--cols:" + cols + ";--rows:" + rows + ";";
    const child = document.createElement("canvas");
    child.width = cols * 8;
    child.height = rows * 16;
    child.classList.add("screen");
    child.tabIndex = 0;
    wrapper.appendChild(child);

    const ctx = child.getContext('2d');
    ctx.font = "16px 'unscii-16', monospace";
    ctx.textBaseline = 'top';
    ctx.fillStyle = 'black';
    ctx.imageSmoothingEnabled = false;
    ctx.shadowColor = "transparent";
    ctx.fillRect(0, 0, child.width, child.height);

    element.appendChild(wrapper);

    screens.push({
        element: wrapper,
        ctx: ctx,
        cols: cols,
        rows: rows,
    });
    return screens.length - 1;
}

function getScreenElement(id) {
    return screens[id].element;
}

function setCell(id, x, y, val, fg, bg) {
    const screen = screens[id];
    x *= 8;
    y *= 16;
    screen.ctx.fillStyle = '#' + bg.toString(16).padStart(6, "0");
    screen.ctx.fillRect(x, y, 8, 16);
    screen.ctx.fillStyle = '#' + fg.toString(16).padStart(6, "0");
    screen.ctx.fillText(val, x, y);
}

async function addDefaultComputer() {
    const computer = new window.nwComputer();
    const screen = new window.nwScreen(computer, document.getElementById('container'), 1);
    computer.add_eeprom(await fetchFileBytes('luaBios.lua'));
    computer.add_vfs(await fetchFileBytesCompressed('openos.ntar.gz'));
    const screenElement = getScreenElement(screen.id);
    const child = document.createElement("div");
    child.innerText = "Click to run";
    child.classList.add("screen-overlay");
    screenElement.appendChild(child);
    screenElement.onclick = () => {
        const overlay = screenElement.getElementsByTagName("div")[0];
        screenElement.removeChild(overlay);
        computer.start_ticking();
    };
    screenElement.onfocus = screenElement.onclick;
}
