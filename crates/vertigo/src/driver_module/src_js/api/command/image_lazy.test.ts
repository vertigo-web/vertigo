// Pictures resized by a chunk: the calls coming before it wait for it, and are told when it
// doesn't come.

import type { JsJsonType } from "../../jsjson";
import type { ImageOp } from "./image";
import { LazyImages } from "./image_lazy";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

const resize = (callback: number): ImageOp => ({
    Resize: {
        callback,
        data: new Uint8Array([1]),
        max_width: 10,
        max_height: 10,
        mime: 'image/jpeg',
        quality: 0.85,
        background: '#fff',
        keep_fitting: false,
    }
});

async function testWaitsForTheChunk() {
    console.log("\n--- Test lazy image: the calls wait for the chunk ---");
    const ran: Array<number> = [];
    const answers: Array<[number, JsJsonType]> = [];
    let loads = 0;
    let come = (): void => { throw new Error('not loading'); };
    let fail = (): void => { throw new Error('not loading'); };
    const errors: Array<unknown> = [];
    console.error = (...args: Array<unknown>) => { errors.push(args); };

    const lazy = new LazyImages((callback, answer) => { answers.push([callback, answer]); }, () => {
        loads += 1;
        return new Promise((ok, failed) => {
            come = () => { ok({ run: (op) => { ran.push(op.Resize.callback); } }); };
            fail = () => { failed(new Error('offline')); };
        });
    });
    assert(loads === 0, "a page resizing nothing asks for no chunk");

    lazy.run(resize(1));
    lazy.run(resize(2));
    assert(loads === 1 && ran.length === 0, "the first picture loads the chunk once, and waits");

    fail();
    await settled();
    assert(errors.length === 1, "a chunk that fails to load says so");
    assert(
        JSON.stringify(answers) === JSON.stringify([
            [1, { Failed: { message: 'the code for pictures failed to load' } }],
            [2, { Failed: { message: 'the code for pictures failed to load' } }],
        ]),
        `and wasm, waiting for each picture, is told: ${JSON.stringify(answers)}`,
    );

    lazy.run(resize(3));
    assert(loads === 2, "the next picture asks for the chunk again");

    come();
    await settled();
    assert(JSON.stringify(ran) === '[3]', "and is resized once it's in, the ones told of the failure not");

    lazy.run(resize(4));
    assert(JSON.stringify(ran) === '[3,4]' && loads === 2, "the later ones at once");
}

void testWaitsForTheChunk();
