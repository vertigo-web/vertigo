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
        closePath: call('closePath'),
        drawImage: (image: { name: string }, ...args: Array<number>) => {
            log.push(`drawImage(${[image.name, ...args.map(round)].join(',')})`);
        },
        createLinearGradient: (...args: Array<number>) => {
            // As a browser's, which takes finite numbers only, unlike the shapes.
            if (!args.every(isFinite)) {
                throw new TypeError(`createLinearGradient: non-finite ${args.join(',')}`);
            }
            log.push(`createLinearGradient(${args.map(round).join(',')})`);
            return {
                addColorStop: (offset: number, color: string) => {
                    if (!(offset >= 0 && offset <= 1)) {
                        throw new Error(`IndexSizeError: ${offset}`);
                    }
                    if (color === 'no color') {
                        throw new Error(`SyntaxError: ${color}`);
                    }
                    log.push(`gradient.addColorStop(${round(offset)},${color})`);
                },
                toString: () => 'gradient',
            };
        },
        createPattern: (image: { name: string }, repetition: string) => {
            log.push(`createPattern(${image.name},${repetition})`);
            return {
                setTransform: (matrix: Record<string, number>) => {
                    log.push(`pattern.setTransform(${['a', 'b', 'c', 'd', 'e', 'f'].map((key) => round(matrix[key] ?? 0)).join(',')})`);
                },
                toString: () => `pattern(${image.name})`,
            };
        },
    };
    for (const name of ['fillStyle', 'strokeStyle', 'globalAlpha', 'lineWidth', 'lineCap', 'lineJoin']) {
        Object.defineProperty(context, name, property(name));
    }

    return context as unknown as Context2d;
};

// `fixture()` in canvas.rs: a view box of -8, -5, 16 x 10 and one of every operation.
const FIXTURE = new Uint8Array([
    1, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0, 32, 65, 1, 7, 0, 35, 102, 102, 100, 50,
    55, 102, 2, 8, 0, 197, 188, 195, 179, 197, 130, 196, 135, 3, 0, 0, 0, 63, 4, 0, 0, 0, 64, 5, 0,
    0, 192, 63, 6, 1, 12, 2, 7, 0, 0, 0, 193, 0, 0, 160, 192, 0, 0, 128, 65, 0, 0, 32, 65, 8, 0, 0,
    128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 9, 0, 0, 128, 191, 0, 0, 0, 192, 0, 0, 128, 64, 10, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 192, 64, 11, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0,
    128, 63, 0, 0, 0, 64, 0, 0, 0, 0, 13, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 191, 0, 0, 0, 64, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 192, 0, 0, 0, 0, 14, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    128, 191, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 15, 7, 0, 99, 97, 116, 46, 112,
    110, 103, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 0, 0, 128, 191, 0, 0, 0,
    192, 0, 0, 0, 64, 0, 0, 128, 63, 16, 7, 0, 99, 97, 116, 46, 112, 110, 103, 0, 0, 0, 63, 7, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 64, 17, 0, 0, 0, 0, 0, 0, 128, 191, 0, 0, 0, 64, 0,
    0, 128, 63, 2, 0, 0, 0, 0, 0, 3, 0, 114, 101, 100, 0, 0, 128, 63, 4, 0, 35, 48, 48, 102, 7, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 64,
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
    const cat = { name: 'cat' } as unknown as CanvasImageSource;
    play(recording(log), FIXTURE, 320, 200, 2, (url) => url === 'cat.png' ? cat : null);

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
        'beginPath()', 'moveTo(0,-1)', 'lineTo(2,0)', 'lineTo(0,1)', 'lineTo(-2,0)', 'closePath()',
        'fill()',
        'beginPath()', 'moveTo(0,-1)', 'lineTo(2,0)', 'lineTo(0,1)', 'closePath()', 'stroke()',
        'drawImage(cat,1,2,3,4,-1,-2,2,1)',
        'createPattern(cat,repeat)', 'pattern.setTransform(0.5,0,0,0.5,0,0)', 'fillStyle=pattern(cat)',
        'fillRect(0,0,2,2)',
        'createLinearGradient(0,-1,2,1)', 'gradient.addColorStop(0,red)', 'gradient.addColorStop(1,#00f)',
        'fillStyle=gradient', 'fillRect(0,0,2,2)',
    ], 'every operation, in order, in the units of the view box');
}

console.log("\n--- Test canvas: an image not there ---");
{
    const log: Array<string> = [];
    play(recording(log), FIXTURE, 320, 200, 2);
    assert(!log.some((line) => line.startsWith('drawImage')), 'an image not loaded is left out');
    assert(!log.some((line) => line.includes('attern')), 'and so is a pattern of it');
    const filled = log.indexOf('fillRect(0,0,2,2)');
    assert(filled > 0 && log.slice(0, filled).filter((line) => line.startsWith('fillStyle')).length === 2,
        'and the drawing goes on without it, filling with the fill style it had');
}

console.log("\n--- Test canvas: patterns ---");
{
    // units: pixels, then for each of `scales` a pattern of `url` at it and a rectangle filled with it
    const patterned = (url: string, scales: Array<number>): Uint8Array => {
        const name = new TextEncoder().encode(url);
        const one = 1 + 2 + name.length + 4 + 1 + 16;
        const bytes = new Uint8Array(1 + one * scales.length);
        const data = new DataView(bytes.buffer);
        scales.forEach((scale, index) => {
            const at = 1 + one * index;
            bytes[at] = 16;
            data.setUint16(at + 1, name.length, true);
            bytes.set(name, at + 3);
            data.setFloat32(at + 3 + name.length, scale, true);
            bytes[at + 7 + name.length] = 7;
            [0, 0, 4, 4].forEach((value, i) => data.setFloat32(at + 8 + name.length + 4 * i, value, true));
        });
        return bytes;
    };
    const grass = { name: 'grass' };
    let asked = 0;
    const log: Array<string> = [];
    play(recording(log), patterned('grass.webp', [0.25, 0.25, 2]), 100, 100, 1, (url) => {
        asked += 1;
        return url === 'grass.webp' ? grass as unknown as CanvasImageSource : null;
    });
    assert(log.filter((line) => line.startsWith('createPattern')).length === 1 && asked === 1,
        'a pattern is made once for a drawing, however many shapes it fills');
    same(log.filter((line) => line.startsWith('pattern.setTransform')), [
        'pattern.setTransform(0.25,0,0,0.25,0,0)',
        'pattern.setTransform(0.25,0,0,0.25,0,0)',
        'pattern.setTransform(2,0,0,2,0,0)',
    ], 'each fill gets the scale it asked for');
}

console.log("\n--- Test canvas: gradients ---");
{
    // units: pixels, then a gradient between `ends` of `stops` and a rectangle filled with it
    const graded = (ends: Array<number>, stops: Array<[number, string]>): Uint8Array => {
        const encoder = new TextEncoder();
        const parts: Array<number> = [0, 17];
        const float = (value: number) => {
            const four = new Uint8Array(4);
            new DataView(four.buffer).setFloat32(0, value, true);
            parts.push(...four);
        };
        ends.forEach(float);
        parts.push(stops.length & 255, stops.length >> 8);
        for (const [offset, color] of stops) {
            float(offset);
            const name = encoder.encode(color);
            parts.push(name.length & 255, name.length >> 8, ...name);
        }
        parts.push(7);
        [0, 0, 4, 4].forEach(float);
        return new Uint8Array(parts);
    };
    // What the drawing did after its defaults and units.
    const played = (bytes: Uint8Array): Array<string> => {
        const log: Array<string> = [];
        play(recording(log), bytes, 100, 100, 1);
        return log.slice(DEFAULTS(100, 100).length + 1);
    };

    same(played(graded([0, 0, 10, 0], [[-0.5, 'red'], [0.5, 'no color'], [Number.NaN, 'lime'], [2, 'blue']])), [
        'createLinearGradient(0,0,10,0)', 'gradient.addColorStop(0,red)', 'gradient.addColorStop(1,blue)',
        'fillStyle=gradient', 'fillRect(0,0,4,4)',
    ], 'an offset outside 0 to 1 counts as the nearer end, and a stop without one or without a color is left out');

    same(played(graded([5, 5, 5, 5], [[1, 'lime'], [0, 'red'], [1, 'blue'], [1, 'no color']])), [
        'createLinearGradient(5,5,5,5)', 'gradient.addColorStop(1,lime)', 'gradient.addColorStop(0,red)',
        'gradient.addColorStop(1,blue)', 'fillStyle=blue', 'fillRect(0,0,4,4)',
    ], 'a gradient with its ends at one point fills with its last stop, as in SVG');

    same(played(graded([5, 5, 5, 5], [[0.5, 'no color']])), [
        'createLinearGradient(5,5,5,5)', 'fillStyle=gradient', 'fillRect(0,0,4,4)',
    ], 'and with nothing when it took no stop, as any gradient without stops');

    for (const ends of [[0, Number.NaN, 10, 0], [0, 0, Number.POSITIVE_INFINITY, 0]]) {
        same(played(graded(ends, [[0, 'red'], [1, 'blue']])), ['fillRect(0,0,4,4)'],
            `an end that is not a finite number leaves the fill style as it was: ${ends.join(',')}`);
    }
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

console.log("\n--- Test canvas: polygons of too few points ---");
{
    // units: pixels, then a filled and a stroked polygon of two points each
    const bytes = new Uint8Array(1 + 2 * (1 + 4 + 2 * 8));
    const data = new DataView(bytes.buffer);
    [13, 14].forEach((op, index) => {
        const at = 1 + index * (1 + 4 + 2 * 8);
        bytes[at] = op;
        data.setUint32(at + 1, 2, true);
        data.setFloat32(at + 5 + 8, 1, true);
    });
    const log: Array<string> = [];
    play(recording(log), bytes, 10, 10, 1);
    assert(!log.includes('fill()') && !log.includes('stroke()'), 'two points enclose nothing');
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

console.log("\n--- Test canvas: loading images ---");
{
    class FakeCanvas {
        public width = 300;
        public height = 150;
        public isConnected = true;
        public readonly log: Array<string> = [];
        get clientWidth() { return 160; }
        get clientHeight() { return 100; }
        getContext() { return recording(this.log); }
    }

    // An image the test loads or fails by hand.
    class FakeImage {
        static made: Array<FakeImage> = [];
        public onload: (() => void) | null = null;
        public onerror: (() => void) | null = null;
        public complete = false;
        public naturalWidth = 0;
        public src = '';
        get name() { return this.src; }
        constructor() { FakeImage.made.push(this); }
        load() { this.complete = true; this.naturalWidth = 32; this.onload?.(); }
        fail() { this.complete = true; this.onerror?.(); }
    }

    const global = globalThis as unknown as Record<string, unknown>;
    global['HTMLCanvasElement'] = FakeCanvas;
    global['Image'] = FakeImage;
    global['ResizeObserver'] = undefined;
    global['window'] = { devicePixelRatio: 1 };

    // units: pixels, then the part 0, 0, 32 x 16 of the image at `url` into 10, 20, 32 x 16
    const image = (url: string): Uint8Array => {
        const name = new TextEncoder().encode(url);
        const bytes = new Uint8Array(1 + 1 + 2 + name.length + 8 * 4);
        const data = new DataView(bytes.buffer);
        bytes[1] = 15;
        data.setUint16(2, name.length, true);
        bytes.set(name, 4);
        [0, 0, 32, 16, 10, 20, 32, 16].forEach((value, index) => {
            data.setFloat32(4 + name.length + 4 * index, value, true);
        });
        return bytes;
    };
    const drew = (canvas: FakeCanvas, url: string) =>
        canvas.log.includes(`drawImage(${url},0,0,32,16,10,20,32,16)`);

    const canvases = new Canvases();
    const draw = (canvas: FakeCanvas, bytes: Uint8Array) => {
        canvases.draw(canvas as unknown as Element, bytes);
        canvases.paintDrawn();
    };

    const first = new FakeCanvas();
    draw(first, image('atlas.webp'));
    assert(FakeImage.made.length === 1 && FakeImage.made[0]?.src === 'atlas.webp', 'the image is loaded');
    assert(first.log.length > 0 && !drew(first, 'atlas.webp'), 'the canvas is drawn without it meanwhile');

    const second = new FakeCanvas();
    draw(second, image('atlas.webp'));
    assert(FakeImage.made.length === 1, 'an image is loaded once, whoever asks for it');

    const gone = new FakeCanvas();
    draw(gone, image('atlas.webp'));
    canvases.forget(gone as unknown as Element);
    gone.log.length = 0;

    FakeImage.made[0]?.load();
    assert(drew(first, 'atlas.webp') && drew(second, 'atlas.webp'), 'the canvases waiting are drawn again with it');
    assert(gone.log.length === 0, 'but not a canvas gone meanwhile');

    const third = new FakeCanvas();
    draw(third, image('atlas.webp'));
    assert(drew(third, 'atlas.webp') && FakeImage.made.length === 1, 'an image loaded is drawn at once');

    // An image that fails to load.
    const broken = new FakeCanvas();
    draw(broken, image('missing.webp'));
    const error = console.error;
    const errors: Array<unknown> = [];
    console.error = (...args: Array<unknown>) => { errors.push(args); };
    try {
        FakeImage.made[1]?.fail();
    } finally {
        console.error = error;
    }
    assert(errors.length === 1 && String(errors[0]).includes('missing.webp'), 'an image that fails is logged');
    broken.log.length = 0;
    draw(broken, image('missing.webp'));
    assert(broken.log.length > 0 && !drew(broken, 'missing.webp') && FakeImage.made.length === 2,
        'and left out, without loading it again');

    // An image loaded beforehand.
    canvases.preload('large.webp');
    assert(FakeImage.made.length === 3 && FakeImage.made[2]?.src === 'large.webp', 'an image asked for beforehand is loaded');
    canvases.preload('large.webp');
    canvases.preload('atlas.webp');
    assert(FakeImage.made.length === 3, 'once, and not again if a drawing loaded it');
    const early = new FakeCanvas();
    draw(early, image('large.webp'));
    assert(!drew(early, 'large.webp'), 'a drawing asking for it before it comes is drawn without it');
    FakeImage.made[2]?.load();
    assert(drew(early, 'large.webp'), 'and again with it');
    const later = new FakeCanvas();
    draw(later, image('large.webp'));
    assert(drew(later, 'large.webp') && FakeImage.made.length === 3, 'a drawing asking for it once it came has it at once');
}
