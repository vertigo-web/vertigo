// The bootstrap: the wasm and the runtime asked for together once the page has loaded, or
// while it loads with a preload, and what the frames said meanwhile handed to the runtime.

import type { EarlyMessages } from "./api/command/dom/callbackManager";
import { boot } from "./boot";
import type { Runtime } from "./boot";
import type { WasmDownload } from "./wasm_init";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

// After the promises settled and node told of the ones that failed with nobody waiting. With
// the timer of node, as the page's is a mock.
const nodeSetTimeout = setTimeout;
const settled = () => new Promise((resolve) => nodeSetTimeout(resolve, 0));

const WASM = '/build/app.wasm';

type Listener = (event: unknown) => void;

/// The page with the script of the app, and what the bootstrap asked of the browser.
class MockPage {
    listeners: Map<string, Array<Listener>> = new Map();
    fetched: Array<[string, unknown]> = [];
    responses: Array<Promise<unknown>> = [];
    compiled: Array<unknown> = [];
    timers: Array<[() => void, number]> = [];
    warnings: Array<unknown> = [];

    /// `preload` is the page of `vertigo serve --wasm-preload`, `answer` what a request gets.
    constructor(preload: boolean, answer: () => Promise<unknown>, compile: () => Promise<unknown>) {
        const script = { getAttribute: (name: string) => name === 'data-vertigo-run-wasm' ? WASM : null };

        (globalThis as any).window = {
            addEventListener: (name: string, listener: Listener) => {
                this.listeners.set(name, [...this.heard(name), listener]);
            },
            removeEventListener: (name: string, listener: Listener) => {
                this.listeners.set(name, this.heard(name).filter((other) => other !== listener));
            },
        };
        (globalThis as any).document = {
            querySelectorAll: (selector: string) =>
                preload || !selector.includes('data-vertigo-preload') ? [script] : [],
        };
        (globalThis as any).fetch = (url: string, init: unknown) => {
            this.fetched.push([url, init]);
            const response = answer();
            this.responses.push(response);
            return response;
        };
        (globalThis as any).WebAssembly.compileStreaming = (source: unknown) => {
            this.compiled.push(source);
            return compile();
        };
        (globalThis as any).setTimeout = (callback: () => void, delay: number) => {
            this.timers.push([callback, delay]);
        };
        console.warn = (...args: Array<unknown>) => { this.warnings.push(args); };
    }

    heard(name: string): Array<Listener> {
        return this.listeners.get(name) ?? [];
    }

    fire(name: string, event?: unknown) {
        this.heard(name).forEach((listener) => listener(event));
    }
}

/// The runtime, which comes when the test lets it, and what the bootstrap asked of it.
class MockRuntime {
    loads = 0;
    started: Array<{ wasm: string, download: WasmDownload, heard: EarlyMessages }> = [];
    entered: Array<[number, number]> = [];
    come = (): void => { throw new Error('not loading'); };
    fail = (): void => { throw new Error('not loading'); };

    load = (): Promise<Runtime> => {
        this.loads += 1;
        return new Promise((ok, failed) => {
            this.come = () => {
                ok({
                    start: (wasm, download, heard) => {
                        this.started.push({ wasm, download, heard });
                        return Promise.resolve({
                            vertigoEntryFunction: (major, minor) => { this.entered.push([major, minor]); },
                        });
                    },
                });
            };
            this.fail = () => { failed(new Error('offline')); };
        });
    };
}

/// What node tells of promises that failed with nobody waiting for them, which ends the test
/// otherwise. So `main` tells of a failed assertion itself.
const unhandled: Array<unknown> = [];
const node = (globalThis as any).process;
node.on('unhandledRejection', (reason: unknown) => { unhandled.push(reason); });

async function testAsksForBothOnLoad() {
    console.log("\n--- Test bootstrap 1: the page has loaded ---");
    const response = {};
    const page = new MockPage(false, () => Promise.resolve(response), () => Promise.resolve({}));
    const runtime = new MockRuntime();

    boot(runtime.load, 0, 14);
    assert(page.fetched.length === 0 && runtime.loads === 0, "nothing is asked for while the page loads");
    assert(JSON.stringify(page.timers.map(([, delay]) => delay)) === '[3000]', "but for a timer of 3 s");

    const early = { data: 'early' };
    page.fire('message', early);

    page.fire('load');
    assert(
        JSON.stringify(page.fetched) === JSON.stringify([[WASM, undefined]]) && runtime.loads === 1,
        "once it has, the wasm and the runtime are asked for together",
    );
    assert(runtime.started.length === 0, "and the wasm isn't waited for by anything yet");

    runtime.come();
    await settled();
    const started = runtime.started[0]!;
    assert(runtime.started.length === 1 && started.wasm === WASM, "the runtime starts the wasm when it comes");
    assert(
        started.download.module === undefined && await started.download.response === response,
        "with the answer to the request already made",
    );
    assert(page.compiled.length === 0, "which the bootstrap left for the runtime to compile");
    assert(JSON.stringify(runtime.entered) === '[[0,14]]', "and the app is run with vertigo-cli's version");

    assert(
        started.heard.messages.length === 1 && started.heard.messages[0] === early,
        "what a frame said meanwhile is handed to the runtime",
    );
    started.heard.stop();
    page.fire('message', { data: 'late' });
    assert(started.heard.messages.length === 1, "which stops the bootstrap hearing more");

    page.timers[0]![0]();
    page.fire('load');
    await settled();
    assert(
        page.fetched.length === 1 && runtime.loads === 1 && runtime.started.length === 1,
        "the timer finds the app running, and starts nothing again",
    );
}

async function testPreload() {
    console.log("\n--- Test bootstrap 2: --wasm-preload ---");
    const module = {};
    const page = new MockPage(true, () => Promise.resolve({}), () => Promise.resolve(module));
    const runtime = new MockRuntime();

    boot(runtime.load, 0, 14);
    assert(
        JSON.stringify(page.fetched) === JSON.stringify([[WASM, { priority: 'low' }]]),
        "the wasm is asked for while the page loads, at low priority",
    );
    assert(page.compiled.length === 1 && page.compiled[0] === page.responses[0], "compiled as it comes");
    assert(runtime.loads === 1, "and the runtime too");

    runtime.come();
    await settled();
    assert(runtime.started.length === 0, "the app waits for the page to load");

    page.fire('load');
    await settled();
    const started = runtime.started[0]!;
    assert(page.fetched.length === 1 && runtime.loads === 1, "then nothing is asked for again");
    assert(
        started.download.response === undefined && await started.download.module === module,
        "and the runtime starts the wasm compiled meanwhile",
    );
}

async function testFailures() {
    console.log("\n--- Test bootstrap 3: what fails to come ---");
    let page = new MockPage(true, () => Promise.resolve({}), () => Promise.reject(new Error('aborted')));
    let runtime = new MockRuntime();

    boot(runtime.load, 0, 14);
    runtime.fail();
    await settled();
    assert(page.warnings.length === 1, "a preload that fails says so");
    assert(unhandled.length === 0, "and the runtime that does is not told of, with nobody waiting for it yet");

    page.fire('load');
    await settled();
    assert(unhandled.length === 1, "but once the page has loaded, as the app doesn't start");

    unhandled.length = 0;
    page = new MockPage(true, () => Promise.resolve({}), () => Promise.reject(new Error('aborted')));
    runtime = new MockRuntime();
    boot(runtime.load, 0, 14);
    runtime.come();
    page.fire('load');
    await settled();
    assert(
        await runtime.started[0]!.download.module === null,
        "the runtime is told of a preload that failed, to fetch the wasm itself",
    );

    page = new MockPage(false, () => Promise.reject(new Error('offline')), () => Promise.resolve({}));
    runtime = new MockRuntime();
    boot(runtime.load, 0, 14);
    page.fire('load');
    await settled();
    assert(unhandled.length === 0, "a request that fails before the runtime comes is not told of either");

    runtime.come();
    await settled();
    let failed = false;
    await runtime.started[0]!.download.response!.catch(() => { failed = true; });
    assert(failed, "but the runtime, which waits for it, is");
}

async function main() {
    await testAsksForBothOnLoad();
    await testPreload();
    await testFailures();
}

main().catch((err: unknown) => {
    console.error(err);
    node.exitCode = 1;
});
