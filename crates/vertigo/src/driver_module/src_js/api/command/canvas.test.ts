// Drawings played back on a canvas, against the bytes `crates/vertigo/src/canvas.rs` makes.

import { Canvases, Context2d, play } from "./canvas";

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

const round = (value: number): string => `${Number(value.toFixed(4))}`;

// A 2D context that writes down every call and every property set, in order.
const recording = (log: Array<string>): Context2d => {
    const call = (name: string) => (...args: Array<number>) => {
        log.push(`${name}(${args.map(round).join(',')})`);
    };
    const property = (name: string) => ({
        set: (value: string | number) => {
            log.push(`${name}=${typeof value === 'number' ? round(value) : value}`);
        },
        get: () => '',
    });

    const context = {
        setTransform: call('setTransform'),
        clearRect: call('clearRect'),
        fillRect: call('fillRect'),
        beginPath: call('beginPath'),
        arc: call('arc'),
        moveTo: call('moveTo'),
        lineTo: call('lineTo'),
        fill: call('fill'),
        stroke: call('stroke'),
    };
    for (const name of ['fillStyle', 'strokeStyle', 'globalAlpha', 'lineWidth', 'lineCap', 'lineJoin']) {
        Object.defineProperty(context, name, property(name));
    }

    return context as unknown as Context2d;
};

// `fixture()` in canvas.rs: a view box of -8, -5, 16 x 10 and one of every operation.
const FIXTURE = new Uint8Array([
    1, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0, 32, 65, 1, 7, 0, 35, 102, 102, 100,
    50, 55, 102, 2, 8, 0, 197, 188, 195, 179, 197, 130, 196, 135, 3, 0, 0, 0, 63, 4, 0, 0, 0,
    64, 5, 0, 0, 192, 63, 6, 1, 12, 2, 7, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0, 32, 65,
    8, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 9, 0, 0, 128, 191, 0, 0, 0, 192, 0, 0, 128,
    64, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 64, 11, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 0, 0,
]);

const DEFAULTS = (width: number, height: number) => [
    'setTransform(1,0,0,1,0,0)',
    `clearRect(0,0,${width},${height})`,
    'globalAlpha=1',
    'fillStyle=#000',
    'strokeStyle=#000',
    'lineWidth=1',
    'lineCap=butt',
    'lineJoin=miter',
];

const TAU = round(2 * Math.PI);

console.log("\n--- Test canvas: the cross-language fixture ---");
{
    const log: Array<string> = [];
    // 160 x 100 CSS pixels at two device pixels each: the box fits exactly, 20 device
    // pixels to a unit, so a CSS pixel is a tenth of a unit.
    play(recording(log), FIXTURE, 320, 200, 2);

    same(log, [
        ...DEFAULTS(320, 200),
        'setTransform(20,0,0,20,160,100)',
        'fillStyle=#ffd27f',
        'strokeStyle=żółć',
        'globalAlpha=0.5',
        'lineWidth=2',
        'lineWidth=0.15',
        'lineCap=round',
        'lineJoin=bevel',
        'fillRect(-8,-5,16,10)',
        'beginPath()', `arc(1,2,3,0,${TAU})`, 'fill()',
        'beginPath()', `arc(-1,-2,0.4,0,${TAU})`, 'fill()',
        'beginPath()', `arc(0,0,6,0,${TAU})`, 'stroke()',
        'beginPath()', 'moveTo(0,0)', 'lineTo(1,1)', 'lineTo(2,0)', 'stroke()',
    ], 'every operation, in order, in the units of the view box');
}

console.log("\n--- Test canvas: fitting the view box ---");
{
    const log: Array<string> = [];
    // Wider than the box: the height decides, and the box stands in the middle.
    play(recording(log), FIXTURE, 400, 200, 1);
    assert(log[8] === 'setTransform(20,0,0,20,200,100)', `a wide canvas centers the box: ${log[8]}`);
    // A CSS pixel is now a twentieth of a unit.
    assert(log.includes(`arc(-1,-2,0.2,0,${TAU})`), 'a radius in pixels follows the scale');
}

const pixels = (...values: Array<number>): Uint8Array => {
    // units: pixels, then a filled circle
    const bytes = new Uint8Array(2 + 4 * values.length);
    const data = new DataView(bytes.buffer);
    bytes[0] = 0;
    bytes[1] = 8;
    values.forEach((value, index) => data.setFloat32(2 + 4 * index, value, true));
    return bytes;
};

console.log("\n--- Test canvas: CSS pixels ---");
{
    const log: Array<string> = [];
    play(recording(log), pixels(10, 20, 3), 100, 100, 2);
    same(log, [
        ...DEFAULTS(100, 100),
        'setTransform(2,0,0,2,0,0)',
        'beginPath()', `arc(10,20,3,0,${TAU})`, 'fill()',
    ], 'units are CSS pixels from the top left corner');
}

console.log("\n--- Test canvas: a bad operation ---");
{
    let thrown = '';
    try {
        play(recording([]), new Uint8Array([0, 200]), 10, 10, 1);
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('unknown operation 200'), 'an unknown operation is an error');
}

console.log("\n--- Test canvas: size and resize ---");
{
    // On the page, 160 x 100 CSS pixels, unless it is as big as its bitmap.
    class FakeCanvas {
        public width = 300;
        public height = 150;
        public isConnected = true;
        public css: { width: number, height: number } | null = { width: 160, height: 100 };
        public readonly log: Array<string> = [];
        get clientWidth() { return this.isConnected ? this.css?.width ?? this.width : 0; }
        get clientHeight() { return this.isConnected ? this.css?.height ?? this.height : 0; }
        getContext() { return recording(this.log); }
    }

    // What the observer calls with the sizes it sees.
    const watchers: Array<(entries: Array<unknown>) => void> = [];
    const observed: Array<unknown> = [];
    const boxes: Array<unknown> = [];
    const unobserved: Array<unknown> = [];

    class FakeObserver {
        constructor(callback: (entries: Array<unknown>) => void) { watchers.push(callback); }
        observe(target: unknown, options?: { box?: string }) { observed.push(target); boxes.push(options?.box); }
        unobserve(target: unknown) { unobserved.push(target); }
    }

    const global = globalThis as unknown as Record<string, unknown>;
    global['HTMLCanvasElement'] = FakeCanvas;
    global['ResizeObserver'] = FakeObserver;
    const page = { devicePixelRatio: 2 };
    global['window'] = page;

    const resized = (canvas: FakeCanvas, width: number, height: number) => {
        watchers[0]?.([{ target: canvas, contentRect: { width, height } }]);
    };

    const canvases = new Canvases();
    // The end of a batch of DOM changes.
    const batchApplied = () => canvases.paintDrawn();

    // What `console.error` got while `run` ran.
    const logged = (run: () => void): Array<unknown> => {
        const error = console.error;
        const errors: Array<unknown> = [];
        console.error = (...args: Array<unknown>) => { errors.push(args); };
        try {
            run();
        } finally {
            console.error = error;
        }
        return errors;
    };

    const canvas = new FakeCanvas();
    const log = canvas.log;
    const drawing = pixels(10, 20, 3);

    canvases.draw(canvas as unknown as Element, drawing);
    assert(canvas.width === 300 && log.length === 0, 'drawn only once the batch is in place');
    batchApplied();
    assert(canvas.width === 320 && canvas.height === 200, 'the bitmap is the size on the screen times the pixel ratio');
    assert(log.includes(`arc(10,20,3,0,${TAU})`), 'drawn then');
    log.length = 0;
    batchApplied();
    assert(log.length === 0, 'and not again with the next batch');
    assert(observed.length === 1 && observed[0] === canvas, 'its size is watched');
    assert(boxes[0] === 'device-pixel-content-box', 'in device pixels, which a sharper screen changes too');

    // Whatever the batch the bytes came in does to them afterwards, the canvas keeps its own.
    drawing.fill(0);
    log.length = 0;
    canvas.css = { width: 50, height: 40 };
    resized(canvas, 50, 40);
    assert(canvas.width === 100 && canvas.height === 80, 'a new size makes a new bitmap');
    assert(log.includes('clearRect(0,0,100,80)') && log.includes(`arc(10,20,3,0,${TAU})`),
        'and the last drawing is drawn on it again');

    log.length = 0;
    canvas.css = { width: 500, height: 500 };
    canvases.draw(canvas as unknown as Element, pixels(1, 2, 3));
    batchApplied();
    assert(observed.length === 1, 'a canvas is watched once however often it is drawn');
    assert(canvas.width === 100 && log.includes('clearRect(0,0,100,80)'),
        'the size seen last is used, without measuring again');

    // The pixels the page's ratio says, whatever the browser counts on its screen.
    log.length = 0;
    canvas.css = { width: 75.6, height: 40 };
    watchers[0]?.([{ target: canvas, contentRect: { width: 75.6, height: 40 }, devicePixelContentBoxSize: [{ inlineSize: 76, blockSize: 40 }] }]);
    assert(canvas.width === 151 && canvas.height === 80, `the CSS size times the ratio: ${canvas.width}x${canvas.height}`);
    assert(log.includes('clearRect(0,0,151,80)'), 'and the drawing on it');

    // A browser that watches only the CSS size tells nothing when just the ratio changes.
    log.length = 0;
    page.devicePixelRatio = 1;
    canvases.draw(canvas as unknown as Element, pixels(1, 2, 3));
    batchApplied();
    assert(canvas.width === 151 && log.includes('setTransform(2,0,0,2,0,0)'),
        'a drawing goes on at the ratio its bitmap was sized by');
    page.devicePixelRatio = 2;

    canvases.forget(canvas as unknown as Element);
    assert(unobserved.length === 1 && unobserved[0] === canvas, 'a canvas gone is not watched any more');
    log.length = 0;
    resized(canvas, 10, 10);
    assert(log.length === 0 && canvas.width === 151, 'nor drawn again');

    let thrown = '';
    try {
        canvases.draw({} as Element, pixels(1, 2, 3));
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('not a canvas'), 'a drawing for anything else is an error');

    // A drawing comes in the batch that makes its canvas, before the canvas is on the page.
    const coming = new FakeCanvas();
    coming.isConnected = false;
    canvases.draw(coming as unknown as Element, pixels(10, 20, 3));
    batchApplied();
    assert(coming.width === 300 && coming.height === 150 && coming.log.length === 0,
        'a canvas off the page is neither measured nor drawn');
    coming.isConnected = true;
    resized(coming, 160, 100);
    assert(coming.width === 320 && coming.log.includes(`arc(10,20,3,0,${TAU})`),
        'the observer draws it once it has a size');

    // Hidden, a canvas has no size: it keeps its bitmap until it has one again.
    coming.log.length = 0;
    resized(coming, 0, 0);
    assert(coming.width === 320 && coming.height === 200 && coming.log.length === 0,
        'a canvas of no size keeps its bitmap');

    // Without a size in CSS a canvas is as big as its bitmap: at a ratio of 2 every drawing
    // would make it twice as big.
    const loose = new FakeCanvas();
    loose.css = null;
    canvases.draw(loose as unknown as Element, pixels(10, 20, 3));
    batchApplied();
    assert(loose.width === 300 && loose.height === 150, `it keeps its size: ${loose.width}x${loose.height}`);
    assert(loose.log.includes('setTransform(1,0,0,1,0,0)') && loose.log.includes('clearRect(0,0,300,150)'),
        'drawn a pixel to a CSS pixel');
    resized(loose, 300, 150);
    canvases.draw(loose as unknown as Element, pixels(10, 20, 3));
    batchApplied();
    assert(loose.width === 300 && loose.height === 150, 'and keeps it');

    // A drawing that fails leaves the other canvases to their sizes.
    const broken = new FakeCanvas();
    const fine = new FakeCanvas();
    canvases.draw(broken as unknown as Element, new Uint8Array([0, 200]));
    canvases.draw(fine as unknown as Element, pixels(10, 20, 3));
    const failed = logged(batchApplied);
    assert(failed.length === 1 && String(failed[0]).includes('unknown operation 200'),
        'a drawing that fails is logged');
    assert(fine.width === 320 && fine.log.includes(`arc(10,20,3,0,${TAU})`),
        'and the next canvas of the batch is drawn');
    broken.css = fine.css = { width: 40, height: 30 };
    const errors = logged(() => watchers[0]?.([
        { target: broken, contentRect: { width: 40, height: 30 } },
        { target: fine, contentRect: { width: 40, height: 30 } },
    ]));
    assert(errors.length === 1 && fine.width === 80 && fine.height === 60,
        'a drawing that fails on a new size is logged, and the next canvas gets its own');

    // In the first batch, a canvas is on the page before the styles the app made: until the
    // batch is in place, CSS gives it no size.
    const styled = new FakeCanvas();
    styled.css = null;
    canvases.draw(styled as unknown as Element, pixels(10, 20, 3));
    styled.css = { width: 160, height: 100 };
    batchApplied();
    assert(styled.width === 320 && styled.height === 200,
        `a canvas drawn in the batch that styles it gets the size of its styles: ${styled.width}x${styled.height}`);

    const gone = new FakeCanvas();
    canvases.draw(gone as unknown as Element, pixels(10, 20, 3));
    canvases.forget(gone as unknown as Element);
    batchApplied();
    assert(gone.width === 300 && gone.log.length === 0, 'a canvas removed in the batch that drew it is not drawn');
}
