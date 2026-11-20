// Canvases loaded as a chunk: what comes before it waits for it.

import { CanvasesApi, LazyCanvases } from "./canvas_lazy";

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

// Canvases writing down what they are asked.
const recording = (log: Array<string>): CanvasesApi => ({
    draw: (node, bytes) => { log.push(`draw ${nameOf(node)} ${bytes.join(',')}`); },
    paintDrawn: () => { log.push('paintDrawn'); },
    preload: (url) => { log.push(`preload ${url}`); },
    forget: (node) => { log.push(`forget ${nameOf(node)}`); },
});

// A chunk loading until `resolve` or `reject`.
const chunk = () => {
    const log: Array<string> = [];
    let loads = 0;
    let resolve = (): void => { throw new Error('not loading'); };
    let reject = (): void => { throw new Error('not loading'); };

    const canvases = new LazyCanvases(() => {
        loads += 1;
        return new Promise((ok, failed) => {
            resolve = () => ok(recording(log));
            reject = () => failed(new Error('offline'));
        });
    });

    return { canvases, log, loads: () => loads, resolve: () => resolve(), reject: () => reject() };
};

// Lets the promises settled so far run their callbacks.
const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

// What `console.error` got while `run` ran.
const logged = async (run: () => Promise<void>): Promise<Array<unknown>> => {
    const error = console.error;
    const errors: Array<unknown> = [];
    console.error = (...args: Array<unknown>) => { errors.push(args); };
    try {
        await run();
    } finally {
        console.error = error;
    }
    return errors;
};

async function testWaitsForTheChunk() {
    console.log("\n--- Test canvas chunk: drawings wait for it ---");
    const { canvases, log, loads, resolve } = chunk();

    canvases.paintDrawn();
    canvases.forget(canvas('a'));
    assert(loads() === 0, 'a page without drawings loads no chunk');

    const [a, b, gone] = [canvas('a'), canvas('b'), canvas('gone')];
    const bytes = new Uint8Array([1, 2]);
    canvases.draw(a, bytes);
    // `bytes` is a view into wasm memory, which the next batch overwrites.
    bytes.set([7, 7]);
    canvases.draw(b, new Uint8Array([3]));
    canvases.draw(b, new Uint8Array([4]));
    canvases.draw(gone, new Uint8Array([5]));
    canvases.forget(gone);
    canvases.preload('/cat.png');
    canvases.paintDrawn();
    assert(loads() === 1, 'the first drawing loads the chunk, once');
    same(log, [], 'and nothing is drawn before it is in');

    resolve();
    await settled();
    same(log, ['preload /cat.png', 'draw a 1,2', 'draw b 4', 'paintDrawn'],
        'then the last drawing of each canvas still on the page is drawn, with the images asked for');

    log.length = 0;
    canvases.draw(a, new Uint8Array([6]));
    canvases.preload('/dog.png');
    canvases.forget(a);
    canvases.paintDrawn();
    same(log, ['draw a 6', 'preload /dog.png', 'forget a', 'paintDrawn'], 'and the canvases get the rest');
    assert(loads() === 1, 'from the chunk loaded');
}

async function testPrefetch() {
    console.log("\n--- Test canvas chunk: loaded before a drawing ---");
    const { canvases, log, loads, resolve } = chunk();

    canvases.prefetch();
    canvases.prefetch();
    assert(loads() === 1, 'a canvas the server sent loads the chunk, once');

    resolve();
    await settled();
    same(log, ['paintDrawn'], 'with nothing to draw yet');

    log.length = 0;
    canvases.draw(canvas('a'), new Uint8Array([1]));
    same(log, ['draw a 1'], 'and the first drawing goes to the canvases at once');
}

async function testFailedLoad() {
    console.log("\n--- Test canvas chunk: failed to load ---");
    const { canvases, log, loads, resolve, reject } = chunk();

    canvases.draw(canvas('a'), new Uint8Array([1]));
    const errors = await logged(async () => {
        reject();
        await settled();
    });
    assert(errors.length === 1, 'a chunk that fails to load is logged');

    canvases.draw(canvas('b'), new Uint8Array([2]));
    assert(loads() === 2, 'and the next drawing asks for it again');

    resolve();
    await settled();
    same(log, ['draw a 1', 'draw b 2', 'paintDrawn'], 'with the drawings that waited for it');
}

function testNotACanvas() {
    console.log("\n--- Test canvas chunk: not a canvas ---");
    const { canvases, loads } = chunk();

    let thrown = '';
    try {
        canvases.draw({} as Element, new Uint8Array([1]));
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('not a canvas'), 'a drawing for another element is an error at once');
    assert(loads() === 0, 'which loads no chunk');
}

async function main() {
    await testWaitsForTheChunk();
    await testPrefetch();
    await testFailedLoad();
    testNotACanvas();
}

void main();
