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

async function addDefaultComputer() {
    const computer = new window.nwComputer();
    const screen = new window.nwScreen(computer, document.getElementById('container'), 1);
    computer.add_eeprom(await fetchFileBytes('luaBios.lua'));
    computer.add_vfs(await fetchFileBytesCompressed('openos.ntar.gz'));
    screen.addRunOverlay(computer);
}
