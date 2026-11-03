import { WasmModule } from "./wasm_module";
import { preloadModule } from "./wasm_init";
import { watchPage } from "./api/command/dom/map_nodes";

// vertigo-cli compatibility version, change together with package version.
const VERTIGO_COMPAT_VERSION_MAJOR = 0;
const VERTIGO_COMPAT_VERSION_MINOR = 14;

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

const findAndRunModule = () => {
    document.querySelectorAll('*[data-vertigo-run-wasm]').forEach((node) => {
        const wasm = node.getAttribute('data-vertigo-run-wasm');

        if (typeof wasm === 'string') {
            // A failure goes to the console as an unhandled rejection.
            void runModule(wasm);
        } else {
            console.error('Run error', node);
        }
    });
};

/// The page is a copy served from another site - a translating proxy, a web archive - when its
/// `<base>` points back to the site it came from. The app would ask that site for its data,
/// which the browser's same-origin rules don't allow, while its links stayed with the copy - so
/// the page is left the way the server rendered it.
const servedElsewhere = (): boolean => {
    try {
        return new URL(document.baseURI).origin !== window.location.origin;
    } catch {
        return false;
    }
};

(() => {
    if (servedElsewhere()) {
        console.info(`Wasm module: not started, the page is a copy of ${document.baseURI} served by ${window.location.origin}`);
        return;
    }

    // what else changes the page before the app starts, hydration has to know
    watchPage();

    document.querySelectorAll('*[data-vertigo-run-wasm][data-vertigo-preload]').forEach((node) => {
        const wasm = node.getAttribute('data-vertigo-run-wasm');

        if (typeof wasm === 'string') {
            preloadModule(wasm);
        }
    });

    window.addEventListener('load', findAndRunModule);
    setTimeout(findAndRunModule, 3000);
})();
