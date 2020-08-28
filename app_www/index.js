//@ts-check

import * as wasm from "../build_wasm/app_rust";

console.info('startuje główny moduł js');

wasm.start_app();