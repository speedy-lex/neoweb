let screens = [];

function createScreen(element, cols, rows) {
    let child = document.createElement("div");
    child.classList.add("console");
    child.style = "--cols:" + cols + ";--rows:" + rows + ";"
    for (let i = 0; i < cols * rows; i++) {
        const cell = document.createElement("div");
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
    screens[id].element
}

function setCell(id, x, y, val) {
    const i = x + 80 * y;
    screens[id].element.children[i].innerText = val
}