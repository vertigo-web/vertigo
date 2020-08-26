//@ts-check

import * as wasm from "../build_wasm/app_rust";

//@ts-ignore
window.consoleLog = (message) => {
    console.info(`consoleLog => ${message}`);
};

console.info('startuje główny moduł js');

wasm.startApp();

