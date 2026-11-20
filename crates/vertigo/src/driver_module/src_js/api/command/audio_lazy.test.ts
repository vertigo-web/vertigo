// Sound loaded as a chunk: the calls coming before it wait for it, all of them, in order.

import type { AudioOp } from "./audio";
import { LazyAudio } from "./audio_lazy";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

async function testWaitsForTheChunk() {
    console.log("\n--- Test lazy audio: the calls wait for the chunk ---");
    const ran: Array<AudioOp> = [];
    let loads = 0;
    let resolve = (): void => { throw new Error('not loading'); };
    let reject = (): void => { throw new Error('not loading'); };
    const errors: Array<unknown> = [];
    console.error = (...args: Array<unknown>) => { errors.push(args); };

    const lazy = new LazyAudio(() => {}, () => {
        loads += 1;
        return new Promise((ok, failed) => {
            resolve = () => { ok({ run: (op) => { ran.push(op); } }); };
            reject = () => { failed(new Error('offline')); };
        });
    });

    lazy.run({ Listen: { callback: 1 } });
    lazy.run({ LoadSound: { id: 1, url: '/a.ogg' } });
    assert(loads === 1 && ran.length === 0, "the first call loads the chunk once, and waits");

    reject();
    await settled();
    assert(errors.length === 1, "a chunk that fails to load says so");
    lazy.run({ PlaySound: { id: 1, volume: 1 } });
    assert(loads === 2, "and is asked for again by the next call");

    resolve();
    await settled();
    assert(
        JSON.stringify(ran) === JSON.stringify([
            { Listen: { callback: 1 } },
            { LoadSound: { id: 1, url: '/a.ogg' } },
            { PlaySound: { id: 1, volume: 1 } },
        ]),
        "once it's in, the calls run in the order they came",
    );

    lazy.run({ Muted: { muted: true } });
    assert(ran.length === 4 && loads === 2, "and the later ones at once");
}

async function main() {
    await testWaitsForTheChunk();
}

void main();
