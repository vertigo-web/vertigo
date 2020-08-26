//@ts-check

import * as wasm from "../build_wasm/app_rust";


window.consoleLog = (message) => {
    console.info(`consoleLog => ${message}`);
};

console.info('startuje główny moduł js');

wasm.startApp();

