// WebGL loaded as a chunk: the calls coming before it wait for it, all of them, in order.

import { LazyWebGls, WebGlsApi } from "./webgl_lazy";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const same = (got: Array<string>, want: Array<string>, message: string) => {
    const equal = JSON.stringify(got) === JSON.stringify(want);
    if (!equal) {
        console.error('got ', got);
        console.error('want', want);
    }
    assert(equal, message);
};

class FakeCanvas {
    constructor(public readonly name: string) {}
}

(globalThis as unknown as Record<string, unknown>)['HTMLCanvasElement'] = FakeCanvas;

const canvas = (name: string): Element => new FakeCanvas(name) as unknown as Element;

const nameOf = (node: Element | Comment | Text): string =>
    node instanceof FakeCanvas ? node.name : 'not a canvas';

// A chunk loading until `resolve` or `reject`, whose WebGL writes down what it is asked.
const chunk = () => {
    const log: Array<string> = [];
    let loads = 0;
    let resolve = (): void => { throw new Error('not loading'); };
    let reject = (): void => { throw new Error('not loading'); };

    const webgls: WebGlsApi = {
        run: (node, bytes) => { log.push(`run ${nameOf(node)} ${bytes.join(',')}`); },
        forget: (node) => { log.push(`forget ${nameOf(node)}`); },
    };
    const lazy = new LazyWebGls(() => {}, () => {
        loads += 1;
        return new Promise((ok, failed) => {
            resolve = () => ok(webgls);
            reject = () => failed(new Error('offline'));
        });
    });

    return { lazy, log, loads: () => loads, resolve: () => resolve(), reject: () => reject() };
};

// Lets the promises settled so far run their callbacks.
const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

async function testWaitsForTheChunk() {
    console.log("\n--- Test webgl chunk: calls wait for it ---");
    const { lazy, log, loads, resolve } = chunk();

    lazy.forget(canvas('a'));
    assert(loads() === 0, 'a page without WebGL loads no chunk');

    const [a, b, gone] = [canvas('a'), canvas('b'), canvas('gone')];
    const bytes = new Uint8Array([1, 2]);
    lazy.run(a, bytes);
    // `bytes` is a view into wasm memory, which the next batch overwrites.
    bytes.set([7, 7]);
    lazy.run(b, new Uint8Array([3]));
    lazy.run(a, new Uint8Array([4]));
    lazy.run(gone, new Uint8Array([5]));
    lazy.forget(gone);
    assert(loads() === 1, 'the first calls load the chunk, once');
    same(log, [], 'and nothing is played before it is in');

    resolve();
    await settled();
    same(log, ['run a 1,2', 'run a 4', 'run b 3'],
        'then every call of each canvas still on the page is played, in order');

    log.length = 0;
    lazy.run(a, new Uint8Array([6]));
    lazy.forget(a);
    same(log, ['run a 6', 'forget a'], 'and the rest go at once');
    assert(loads() === 1, 'to the chunk loaded');
}

async function testFailedLoad() {
    console.log("\n--- Test webgl chunk: failed to load ---");
    const { lazy, log, loads, resolve, reject } = chunk();

    lazy.run(canvas('a'), new Uint8Array([1]));
    const error = console.error;
    const errors: Array<unknown> = [];
    console.error = (...args: Array<unknown>) => { errors.push(args); };
    try {
        reject();
        await settled();
    } finally {
        console.error = error;
    }
    assert(errors.length === 1, 'a chunk that fails to load is logged');

    lazy.run(canvas('a'), new Uint8Array([2]));
    assert(loads() === 2, 'and the next calls ask for it again');

    resolve();
    await settled();
    same(log, ['run a 1', 'run a 2'], 'with the calls that waited for it');
}

function testNotACanvas() {
    console.log("\n--- Test webgl chunk: not a canvas ---");
    const { lazy, loads } = chunk();

    let thrown = '';
    try {
        lazy.run({} as Element, new Uint8Array([1]));
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('not a canvas'), 'calls for another element are an error at once');
    assert(loads() === 0, 'which load no chunk');
}

async function main() {
    await testWaitsForTheChunk();
    await testFailedLoad();
    testNotACanvas();
}

void main();
