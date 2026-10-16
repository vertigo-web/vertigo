// The frames the browser draws: asked for while wasm wants them, and one call into wasm for each.

import { AnimationFrames } from "./animationFrames";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

// --- MOCKS ---

/// The callbacks asked for the next frame, by request id.
const requested = new Map<number, (time: number) => void>();
let nextRequest = 1;

(globalThis as any).requestAnimationFrame = (callback: (time: number) => void) => {
    const id = nextRequest++;
    requested.set(id, callback);
    return id;
};
(globalThis as any).cancelAnimationFrame = (id: number) => {
    requested.delete(id);
};

/// The browser drawing a frame: it calls what was asked for before the frame, and what these ask
/// for waits for the next one.
const drawFrame = (time: number) => {
    const callbacks = [...requested.values()];
    requested.clear();
    for (const callback of callbacks) {
        callback(time);
    }
};

/// The times of the frames wasm got, and what wasm does on a frame.
const frames: Array<number> = [];
let onFrame = () => {};
const wasm = {
    wasmCommand: (command: any) => {
        frames.push(command.AnimationFrame.time);
        onFrame();
        return null;
    },
};

const driver = new AnimationFrames(() => wasm as any);

const reset = () => {
    driver.stop();
    frames.length = 0;
    onFrame = () => {};
};

// --- TESTS ---

function testNoFramesUntilWasmAsks() {
    console.log("\n--- Test frames 1: nobody asked ---");
    drawFrame(16);
    assert(requested.size === 0, "the browser isn't asked for frames");
    assert(frames.length === 0, "and wasm gets none");
}

function testEveryFrameReachesWasmWithItsTime() {
    console.log("\n--- Test frames 2: wasm asked ---");
    driver.start();
    assert(requested.size === 1, "the browser is asked for the next frame");

    drawFrame(16.5);
    drawFrame(33);
    assert(frames.join() === '16.5,33', `wasm gets every frame with its time: ${frames.join()}`);
    assert(requested.size === 1, "and the frame after them is asked for");
    reset();
}

function testStartingTwiceAsksOnce() {
    console.log("\n--- Test frames 3: asked twice ---");
    driver.start();
    driver.start();
    assert(requested.size === 1, "one frame is asked for");

    drawFrame(16);
    assert(frames.join() === '16', "and wasm gets it once");
    reset();
}

function testStopCancelsTheFrameAskedFor() {
    console.log("\n--- Test frames 4: wasm stops them ---");
    driver.start();
    drawFrame(16);
    driver.stop();
    assert(requested.size === 0, "the frame asked for is cancelled");

    drawFrame(33);
    assert(frames.join() === '16', "and wasm gets no more");
    reset();
}

function testStopFromAFrame() {
    console.log("\n--- Test frames 5: the last callback stops them in a frame ---");
    driver.start();
    onFrame = () => driver.stop();

    drawFrame(16);
    assert(requested.size === 0, "the next frame isn't asked for");

    drawFrame(33);
    assert(frames.join() === '16', "and wasm gets no more");
    reset();
}

function testStopAndStartInOneFrame() {
    console.log("\n--- Test frames 6: the last callback goes and a new one comes in a frame ---");
    driver.start();
    onFrame = () => {
        driver.stop();
        driver.start();
    };

    drawFrame(16);
    assert(requested.size === 1, "one next frame is asked for");

    onFrame = () => {};
    drawFrame(33);
    assert(frames.join() === '16,33', "and wasm gets it once");
    reset();
}

testNoFramesUntilWasmAsks();
testEveryFrameReachesWasmWithItsTime();
testStartingTwiceAsksOnce();
testStopCancelsTheFrameAskedFor();
testStopFromAFrame();
testStopAndStartInOneFrame();
