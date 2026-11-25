// Pictures made smaller: what the browser is asked for and what wasm is told, with the browser's
// own part - reading the file, drawing, encoding - mocked.

import type { JsJsonType } from "../../jsjson";
import { Images } from "./image";
import type { ImageOp } from "./image";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

const globals = globalThis as unknown as Record<string, unknown>;

/// What the browser was asked for, in order, and what it gives.
class MockBrowser {
    calls: Array<string> = [];
    closed = 0;
    /// The size of the picture in the file, `null` for a file that is no picture.
    picture: { width: number, height: number } | null = { width: 4000, height: 3000 };
    /// What the canvas encodes to, `null` when it can't.
    encoded: Array<number> | null = [0xff, 0xd8, 0xff];
    context = true;

    constructor() {
        globals['createImageBitmap'] = (blob: Blob) => {
            this.calls.push(`read ${blob.size} B`);
            if (this.picture === null) {
                return Promise.reject(new Error('The source image could not be decoded.'));
            }
            return Promise.resolve({ ...this.picture, close: () => { this.closed += 1; } });
        };

        globals['document'] = {
            createElement: (name: string) => {
                const canvas = { width: 0, height: 0 };
                const context = {
                    set fillStyle(color: string) { browser.calls.push(`fill style ${color}`); },
                    set imageSmoothingQuality(quality: string) { browser.calls.push(`smoothing ${quality}`); },
                    fillRect: (...rect: Array<number>) => { this.calls.push(`fill ${rect.join(' ')}`); },
                    drawImage: (_picture: unknown, ...rect: Array<number>) => {
                        this.calls.push(`draw ${rect.join(' ')} on ${name} ${canvas.width}x${canvas.height}`);
                    },
                };
                return Object.assign(canvas, {
                    getContext: () => this.context ? context : null,
                    toBlob: (done: (blob: Blob | null) => void, mime: string, quality: number) => {
                        this.calls.push(`encode ${mime} ${quality}`);
                        done(this.encoded === null ? null : new Blob([new Uint8Array(this.encoded)]));
                    },
                });
            },
        };

        // eslint-disable-next-line @typescript-eslint/no-this-alias -- for the setters above, whose `this` is the context
        const browser = this;
    }
}

const resize = (how: Partial<ImageOp['Resize']> = {}): ImageOp => ({
    Resize: {
        callback: 7,
        data: new Uint8Array([1, 2, 3, 4, 5]),
        max_width: 1600,
        max_height: 1600,
        mime: 'image/jpeg',
        quality: 0.85,
        background: '#fff',
        keep_fitting: false,
        ...how,
    }
});

/// The picture resized as `op` says, and what wasm was told.
async function told(op: ImageOp): Promise<Array<[number, JsJsonType]>> {
    const answers: Array<[number, JsJsonType]> = [];
    new Images((callback, answer) => { answers.push([callback, answer]); }).run(op);
    await settled();
    return answers;
}

const json = (value: unknown) => JSON.stringify(value, (_key, item: unknown) => item instanceof Uint8Array ? Array.from(item) : item);

async function testScalesDown() {
    console.log("\n--- Test image 1: a photo too big ---");
    const browser = new MockBrowser();

    const answers = await told(resize());
    assert(
        json(browser.calls) === json([
            'read 5 B',
            'fill style #fff',
            'fill 0 0 1600 1200',
            'smoothing high',
            'draw 0 0 1600 1200 on canvas 1600x1200',
            'encode image/jpeg 0.85',
        ]),
        `the browser reads the file, draws it scaled on white and encodes it: ${json(browser.calls)}`,
    );
    assert(
        json(answers) === json([[7, {
            Resized: { width: 1600, height: 1200, original_width: 4000, original_height: 3000, data: [0xff, 0xd8, 0xff] }
        }]]),
        `wasm is told the picture and both sizes through the call's callback: ${json(answers)}`,
    );
    assert(browser.closed === 1, "and the pixels read are let go of");

    browser.picture = { width: 3000, height: 4000 };
    const tall = await told(resize({ max_width: 800, max_height: 600 }));
    assert(json(tall).includes('"width":450,"height":600'), `the proportions are kept by the side that fits less: ${json(tall)}`);

    browser.picture = { width: 5000, height: 2 };
    const thin = await told(resize({ max_width: 100, max_height: 100 }));
    assert(json(thin).includes('"width":100,"height":1'), "and no side comes down to nothing");
}

async function testFits() {
    console.log("\n--- Test image 2: a picture that fits ---");
    const browser = new MockBrowser();
    browser.picture = { width: 800, height: 600 };

    const again = await told(resize());
    assert(
        json(again).includes('"width":800,"height":600') && json(again).includes('"data":[255,216,255]'),
        "it isn't made bigger, and is encoded as asked",
    );

    browser.calls = [];
    const kept = await told(resize({ keep_fitting: true }));
    assert(
        json(kept) === json([[7, { Resized: { width: 800, height: 600, original_width: 800, original_height: 600, data: null } }]]),
        `kept as it is when asked so, wasm is told its size and no data: ${json(kept)}`,
    );
    assert(json(browser.calls) === json(['read 5 B']) && browser.closed === 2, "nothing is drawn or encoded then");

    browser.picture = { width: 4000, height: 3000 };
    const big = await told(resize({ keep_fitting: true }));
    assert(json(big).includes('"width":1600,"height":1200'), "but one too big is scaled down all the same");
}

async function testFormats() {
    console.log("\n--- Test image 3: other formats ---");
    const browser = new MockBrowser();

    await told(resize({ mime: 'image/webp', quality: 0.7, background: 'black' }));
    assert(
        !browser.calls.some((call) => call.startsWith('fill')) && browser.calls.includes('encode image/webp 0.7'),
        `a format with transparency gets no background: ${json(browser.calls)}`,
    );
}

async function testFailures() {
    console.log("\n--- Test image 4: what the browser can't do ---");
    const browser = new MockBrowser();

    browser.picture = null;
    const unread = await told(resize());
    assert(
        json(unread) === json([[7, { Failed: { message: 'The source image could not be decoded.' } }]]),
        `a file that is no picture is told with what the browser said: ${json(unread)}`,
    );

    browser.picture = { width: 100, height: 100 };
    browser.encoded = null;
    const unencoded = await told(resize());
    assert(json(unencoded).includes('"Failed"') && browser.closed === 1, "one it can't encode too, the pixels let go of");

    browser.encoded = [1];
    browser.context = false;
    const undrawn = await told(resize());
    assert(json(undrawn).includes('"Failed"') && browser.closed === 2, "and one it has no canvas for");
}

async function main() {
    await testScalesDown();
    await testFits();
    await testFormats();
    await testFailures();
}

void main();
