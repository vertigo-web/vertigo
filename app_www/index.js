//@ts-check

import * as wasm from "wasm-module-app";


window.consoleLog = (message) => {
    console.info(`consoleLog => ${message}`);
};

window.alertConsole = (message) => {
    console.info(`alert console - ${message}`);
};

window.callFromWasm = (message) => {
    console.info(`Wiadomosc z wasm -> ${message}`);
}

console.info('aaa');

const aa = wasm.startApp("dupa bladaaaaaa333", "66");
console.info(`AA = ${aa}`);


setInterval(() => {
    const nextId = wasm.getNextId();
    console.info(`next id === ${nextId}`);
}, 1000);
