//@ts-check

import * as wasm from "../build_wasm/app_rust";

//@ts-ignore
window.consoleLog = (message) => {
    console.info(`consoleLog => ${message}`);
};

window.click_button = () => {
    wasm.click_button();
};

console.info('startuje główny moduł js');

wasm.start_app();