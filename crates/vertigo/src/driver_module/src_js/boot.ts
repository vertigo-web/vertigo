// The bootstrap: all of vertigo that a page downloads and runs before it has loaded, which is
// what `wasm_run.js` holds. What wasm talks to the browser through is the runtime (`runtime.ts`),
// a chunk of its own asked for together with the wasm once the page has loaded. So the code,
// which grows with what vertigo can do, takes nothing from what the page shows first, the same
// as the wasm.
//
// Nothing here may be imported by the runtime too, but for types: rollup would keep such code
// in `wasm_run.js` for the chunk to import from, and `rollup.config.mjs` fails the build then.

import type { EarlyMessages } from './api/command/dom/callbackManager';
import type { WasmDownload } from './wasm_init';

// What the bootstrap asks of the runtime, see `runtime.ts`.
export interface Runtime {
    start: (wasm: string, download: WasmDownload, heard: EarlyMessages) => Promise<{
        vertigoEntryFunction: (major: number, minor: number) => void,
    }>,
}

// A promise that fails before anybody waits for it is told to the console as unhandled, and
// once more by the one who waits for it later. That one is still told.
const unreported = <T>(promise: Promise<T>): Promise<T> => {
    promise.catch(() => undefined);
    return promise;
};

// `major` and `minor` are vertigo-cli's compatibility version, see `index.ts`.
export const boot = (loadRuntime: () => Promise<Runtime>, major: number, minor: number) => {
    // What the frames of the page say before the runtime is there to hear them, see
    // `hearEarlyMessages`.
    const messages: Array<MessageEvent> = [];
    const keep = (event: MessageEvent) => {
        messages.push(event);
    };
    window.addEventListener('message', keep);
    const heard: EarlyMessages = {
        messages,
        stop: () => window.removeEventListener('message', keep),
    };

    const preloaded: Map<string, Promise<WebAssembly.Module | null>> = new Map();
    let runtime: Promise<Runtime> | null = null;
    const moduleRun: Set<string> = new Set();

    // Downloads and compiles the wasm while the page loads (`vertigo serve --wasm-preload`),
    // at low priority so the images and styles the page shows first take precedence. The
    // runtime comes meanwhile too, for the app to start as soon as the page has loaded.
    const preload = (wasm: string) => {
        runtime ??= unreported(loadRuntime());

        if (preloaded.has(wasm) || typeof WebAssembly.compileStreaming !== 'function') {
            return;
        }

        const module = WebAssembly.compileStreaming(fetch(wasm, { priority: 'low' })).catch((err: unknown) => {
            console.warn("Preloading wasm failed, it will be fetched again once the page has loaded. Original error:\n", err);
            return null;
        });

        preloaded.set(wasm, module);
    };

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

        // The wasm and the runtime are asked for in one go, so neither waits for the other.
        const module = preloaded.get(wasm);
        const download: WasmDownload = module === undefined ? { response: unreported(fetch(wasm)) } : { module };
        runtime ??= loadRuntime();

        const wasmModule = await (await runtime).start(wasm, download, heard);
        console.info(`Wasm module: "${wasm}" -> initialized`);
        wasmModule.vertigoEntryFunction(major, minor);
        console.info(`Wasm module: "${wasm}" -> launched vertigoEntryFunction with version ${major}.${minor}`);
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

    document.querySelectorAll('*[data-vertigo-run-wasm][data-vertigo-preload]').forEach((node) => {
        const wasm = node.getAttribute('data-vertigo-run-wasm');

        if (typeof wasm === 'string') {
            preload(wasm);
        }
    });

    window.addEventListener('load', findAndRunModule);
    setTimeout(findAndRunModule, 3000);
};
