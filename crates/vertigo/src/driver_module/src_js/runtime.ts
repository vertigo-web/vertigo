// The runtime: what wasm talks to the browser through, all of it but the chunks a page loads
// when it first needs them (canvases, the GPU, sound). A chunk too, named `runtime`, which the
// bootstrap asks for together with the wasm, see `boot.ts`.

import { hearEarlyMessages } from './api/command/dom/callbackManager';
import type { EarlyMessages } from './api/command/dom/callbackManager';
import type { WasmDownload } from './wasm_init';
import { WasmModule } from './wasm_module';

// `download` is the wasm as the bootstrap began to download it, `heard` what the frames of the
// page said meanwhile.
export const start = (wasm: string, download: WasmDownload, heard: EarlyMessages): Promise<WasmModule> => {
    hearEarlyMessages(heard);
    return WasmModule.create(wasm, download);
};
