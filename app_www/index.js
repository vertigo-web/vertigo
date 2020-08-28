//@ts-check

import * as wasm from "../build_wasm/app_rust";

window.click_button = () => {
    wasm.click_button();
};

console.info('startuje główny moduł js');

wasm.start_app();