import { WasmModule } from "./wasm_module";

// vertigo-cli compatibility version, change together with package version.
const VERTIGO_COMPAT_VERSION_MAJOR = 0;
const VERTIGO_COMPAT_VERSION_MINOR = 10;

const moduleRun: Set<string> = new Set();

const runModule = async (wasm: string) => {
    if (moduleRun.has(wasm)) {
        //ok, module is run
        return;
    }

    if (moduleRun.size > 0) {
        console.error('Only one wasm module can be run', { moduleRun, wasm });
        return;
    }

    moduleRun.add(wasm);

    console.info(`Wasm module: "${wasm}" -> start`);
    const wasmModule = await WasmModule.create(wasm);
    console.info(`Wasm module: "${wasm}" -> initialized`);
    wasmModule.vertigoEntryFunction(VERTIGO_COMPAT_VERSION_MAJOR, VERTIGO_COMPAT_VERSION_MINOR);
    console.info(`Wasm module: "${wasm}" -> launched vertigoEntryFunction with version ${VERTIGO_COMPAT_VERSION_MAJOR}.${VERTIGO_COMPAT_VERSION_MINOR}`);
};

const findAndRunModule = async () => {
    document.querySelectorAll('*[data-vertigo-run-wasm]').forEach((node) => {
        const wasm = node.getAttribute('data-vertigo-run-wasm');

        if (typeof wasm === 'string') {
            runModule(wasm);
        } else {
            console.error('Run error', node);
        }
    });
};

(() => {
    window.addEventListener('load', findAndRunModule);
    setTimeout(findAndRunModule, 3000);
})();
