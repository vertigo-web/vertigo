// WebGL calls played back on a context, against the bytes `crates/vertigo/src/webgl.rs` makes.

import { Gl, Objects, WebGls, attributes, play } from "./webgl";

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

// A value as the log shows it: an object WebGL made by its name, bytes and numbers as a list.
const shown = (value: unknown): string => {
    if (value instanceof Uint8Array || value instanceof Float32Array) {
        return `[${Array.from(value).join(',')}]`;
    }
    if (typeof value === 'object' && value !== null && 'name' in value) {
        return String(value.name);
    }
    return String(value);
};

// A context writing down each call into `log`. The objects it makes are named by what they are
// and a number; shaders compile and programs link unless their source says `broken`.
const recording = (log: Array<string>): Gl => {
    let made = 0;
    const sources: Map<unknown, string> = new Map();
    const object = (kind: string) => ({ name: `${kind}${++made}` });

    const answers: Record<string, (...args: Array<unknown>) => unknown> = {
        createBuffer: () => object('buffer'),
        createVertexArray: () => object('array'),
        createProgram: () => object('program'),
        createShader: () => object('shader'),
        createTexture: () => object('texture'),
        shaderSource: (shader, source) => sources.set(shader, String(source)),
        getShaderParameter: (shader) => !(sources.get(shader) ?? '').includes('broken'),
        getShaderInfoLog: () => 'ERROR: 0:1: broken',
        getProgramParameter: () => true,
        getUniformLocation: (_program, name) => ({ name: `@${String(name)}` }),
    };

    return new Proxy({}, {
        get: (_target, name: string) => (...args: Array<unknown>) => {
            log.push(`${name}(${args.map(shown).join(',')})`);
            return answers[name]?.(...args);
        },
    }) as Gl;
};

// `fixture()` in webgl.rs: one call of each.
const FIXTURE = new Uint8Array([
    1, 63, 2, 1, 0, 0, 0, 4, 146, 136, 0, 0, 1, 0, 0, 0, 5, 146, 136, 0, 0, 228, 136, 0, 0, 8,
    0, 0, 0, 0, 0, 192, 63, 0, 0, 0, 192, 6, 147, 136, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 7, 0, 8,
    0, 7, 2, 0, 0, 0, 9, 2, 0, 0, 0, 10, 1, 0, 0, 0, 11, 1, 0, 0, 0, 3, 6, 20, 0, 0, 1, 12, 0,
    0, 0, 4, 0, 0, 0, 12, 3, 0, 0, 0, 2, 0, 0, 0, 118, 115, 3, 0, 0, 0, 102, 197, 155, 14, 3,
    0, 0, 0, 15, 1, 0, 97, 0, 0, 0, 63, 16, 1, 0, 98, 0, 0, 128, 63, 0, 0, 0, 64, 17, 1, 0, 99,
    0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 18, 1, 0, 100, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0,
    64, 64, 0, 0, 128, 64, 19, 1, 0, 101, 255, 255, 255, 255, 20, 1, 0, 102, 0, 0, 0, 0, 0, 0,
    128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 224,
    64, 0, 0, 0, 65, 21, 1, 0, 103, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0,
    128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 0, 0, 16, 65, 0, 0, 32,
    65, 0, 0, 48, 65, 0, 0, 64, 65, 0, 0, 80, 65, 0, 0, 96, 65, 0, 0, 112, 65, 22, 4, 0, 0, 0,
    24, 2, 0, 0, 0, 25, 4, 0, 0, 0, 26, 1, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 255, 0, 128, 255,
    27, 1, 40, 0, 0, 0, 38, 0, 0, 27, 0, 40, 0, 0, 1, 38, 0, 0, 27, 2, 40, 0, 0, 1, 41, 0, 0,
    27, 3, 40, 0, 0, 47, 129, 0, 0, 28, 29, 0, 0, 0, 0, 0, 0, 128, 62, 0, 0, 0, 63, 0, 0, 128,
    63, 30, 0, 0, 128, 63, 31, 0, 65, 0, 0, 32, 113, 11, 0, 0, 33, 226, 11, 0, 0, 34, 0, 0, 0,
    0, 255, 255, 255, 255, 128, 2, 0, 0, 224, 1, 0, 0, 35, 3, 2, 0, 0, 36, 2, 3, 0, 0, 3, 3, 0,
    0, 37, 5, 4, 0, 0, 38, 0, 9, 0, 0, 39, 4, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 40, 1, 0, 0, 0,
    6, 0, 0, 0, 3, 20, 0, 0, 2, 0, 0, 0, 4, 146, 136, 0, 0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 8, 2, 0,
    0, 0, 13, 3, 0, 0, 0, 23, 4, 0, 0, 0,
]);

console.log("\n--- Test webgl: the cross-language fixture ---");
{
    const log: Array<string> = [];
    play(recording(log), new Objects(), FIXTURE);

    same(log, [
        'createBuffer()',
        'bindBuffer(34962,buffer1)',
        'bufferData(34962,[0,0,192,63,0,0,0,192],35044)',
        'bufferSubData(34963,2,[7,0,8,0])',
        'createVertexArray()',
        'bindVertexArray(array2)',
        'enableVertexAttribArray(1)',
        'vertexAttribPointer(1,3,5126,true,12,4)',
        'createProgram()',
        'createShader(35633)',
        'shaderSource(shader4,vs)',
        'compileShader(shader4)',
        'getShaderParameter(shader4,35713)',
        'createShader(35632)',
        'shaderSource(shader5,fś)',
        'compileShader(shader5)',
        'getShaderParameter(shader5,35713)',
        'attachShader(program3,shader4)',
        'attachShader(program3,shader5)',
        'linkProgram(program3)',
        'getProgramParameter(program3,35714)',
        'deleteShader(shader4)',
        'deleteShader(shader5)',
        'useProgram(program3)',
        'getUniformLocation(program3,a)',
        'uniform1f(@a,0.5)',
        'getUniformLocation(program3,b)',
        'uniform2f(@b,1,2)',
        'getUniformLocation(program3,c)',
        'uniform3f(@c,1,2,3)',
        'getUniformLocation(program3,d)',
        'uniform4f(@d,1,2,3,4)',
        'getUniformLocation(program3,e)',
        'uniform1i(@e,-1)',
        'getUniformLocation(program3,f)',
        'uniformMatrix3fv(@f,false,[0,1,2,3,4,5,6,7,8])',
        'getUniformLocation(program3,g)',
        'uniformMatrix4fv(@g,false,[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15])',
        'createTexture()',
        'activeTexture(33986)',
        'bindTexture(3553,texture6)',
        'texImage2D(3553,0,32856,1,1,0,6408,5121,[255,0,128,255])',
        'texParameteri(3553,10241,9728)',
        'texParameteri(3553,10240,9729)',
        'texParameteri(3553,10242,10497)',
        'texParameteri(3553,10243,33071)',
        'generateMipmap(3553)',
        'clearColor(0,0.25,0.5,1)',
        'clearDepth(1)',
        'clear(16640)',
        'enable(2929)',
        'disable(3042)',
        'viewport(0,-1,640,480)',
        'depthFunc(515)',
        'blendFunc(770,771)',
        'cullFace(1029)',
        'frontFace(2304)',
        'drawArrays(4,0,3)',
        'drawElements(1,6,5123,2)',
        'bindBuffer(34962,null)',
        'deleteBuffer(buffer1)',
        'deleteVertexArray(array2)',
        'deleteProgram(program3)',
        'deleteTexture(texture6)',
    ], 'every call is played with its arguments, and objects by their ids');

    same(
        Object.entries(attributes(63)).filter(([, on]) => on).map(([name]) => name),
        ['alpha', 'antialias', 'depth', 'stencil', 'premultipliedAlpha', 'preserveDrawingBuffer'],
        'the context gets the attributes it asked for',
    );
    same(Object.values(attributes(4 | 1)).map(String), ['true', 'false', 'true', 'false', 'false', 'false'],
        'and not the others');
}

// Calls as webgl.rs writes them.
const calls = (...ops: Array<Array<number>>): Uint8Array => new Uint8Array(ops.flat());
const u32 = (value: number): Array<number> => [value & 255, (value >> 8) & 255, (value >> 16) & 255, (value >>> 24) & 255];
const name = (text: string): Array<number> => [text.length, 0, ...Array.from(text, (char) => char.charCodeAt(0))];
const source = (text: string): Array<number> => [...u32(text.length), ...Array.from(text, (char) => char.charCodeAt(0))];

console.log("\n--- Test webgl: programs and uniforms ---");
{
    const log: Array<string> = [];
    const gl = recording(log);
    const objects = new Objects();

    const error = console.error;
    const errors: Array<string> = [];
    console.error = (message: unknown) => { errors.push(String(message)); };
    try {
        play(gl, objects, calls([12, ...u32(1), ...source('vs'), ...source('broken')]));
    } finally {
        console.error = error;
    }
    same(errors, ["webgl: the fragment shader doesn't compile:\nERROR: 0:1: broken"],
        "a shader that doesn't compile is told in the console, with WebGL's log");

    log.length = 0;
    play(gl, objects, calls([15, ...name('a'), 0, 0, 128, 63]));
    same(log, ['uniform1f(null,1)'], 'without a program in use, a uniform goes nowhere, which WebGL ignores');
    log.length = 0;

    play(gl, objects, calls([14, ...u32(1)], [15, ...name('a'), 0, 0, 128, 63], [15, ...name('a'), 0, 0, 0, 64]));
    same(log, ['useProgram(program1)', 'getUniformLocation(program1,a)', 'uniform1f(@a,1)', 'uniform1f(@a,2)'],
        'a uniform is found once per program');

    log.length = 0;
    play(gl, objects, calls([13, ...u32(1)], [15, ...name('a'), 0, 0, 128, 63]));
    same(log, ['deleteProgram(program1)', 'uniform1f(null,1)'],
        'and once the program in use is let go, its uniforms go nowhere');

    let thrown = '';
    try {
        play(gl, objects, calls([200]));
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('unknown operation 200'), 'an unknown operation is an error');
}

console.log("\n--- Test webgl: contexts and their size ---");
{
    // On the page, 160 x 100 CSS pixels, unless it is as big as its bitmap.
    class FakeCanvas {
        public width = 300;
        public height = 150;
        public isConnected = true;
        public css: { width: number, height: number } | null = { width: 160, height: 100 };
        public readonly log: Array<string> = [];
        public context: Gl | null = recording(this.log);
        public asked: Array<string> = [];
        get clientWidth() { return this.css?.width ?? this.width; }
        get clientHeight() { return this.css?.height ?? this.height; }
        getContext(type: string, options: WebGLContextAttributes) {
            this.asked.push(`${type} preserveDrawingBuffer=${String(options.preserveDrawingBuffer)}`);
            return this.context;
        }
    }

    const watchers: Array<(entries: Array<unknown>) => void> = [];
    class FakeObserver {
        constructor(callback: (entries: Array<unknown>) => void) { watchers.push(callback); }
        observe() {}
        unobserve() {}
    }

    const global = globalThis as unknown as Record<string, unknown>;
    global['HTMLCanvasElement'] = FakeCanvas;
    global['ResizeObserver'] = FakeObserver;
    global['window'] = { devicePixelRatio: 2 };

    const webgls = new WebGls();
    const run = (canvas: FakeCanvas, bytes: Uint8Array) => webgls.run(canvas as unknown as Element, bytes);
    const clear = calls([31, ...u32(0x4000)]);

    const canvas = new FakeCanvas();
    run(canvas, calls([1, 32], [31, ...u32(0x4000)]));
    same(canvas.asked, ['webgl2 preserveDrawingBuffer=true'], 'the first calls make the context, with its attributes');
    same(canvas.log, ['viewport(0,0,320,200)', 'clear(16384)'],
        'on a bitmap as big as the canvas in device pixels, all of it drawn on');
    assert(canvas.width === 320 && canvas.height === 200, 'which is 320 x 200');

    canvas.log.length = 0;
    run(canvas, clear);
    same(canvas.asked, ['webgl2 preserveDrawingBuffer=true'], 'the context is made once');
    same(canvas.log, ['clear(16384)'], 'and the bitmap is resized only when the canvas is');

    watchers[0]?.([{ target: canvas, contentRect: { width: 200, height: 100 } }]);
    assert(canvas.width === 320, 'a resized canvas keeps its picture until the next calls');
    canvas.log.length = 0;
    run(canvas, clear);
    same(canvas.log, ['viewport(0,0,400,200)', 'clear(16384)'], 'which get a bitmap of the new size');

    // A canvas CSS gives no size is as big as its bitmap.
    const free = new FakeCanvas();
    free.css = null;
    run(free, clear);
    assert(free.width === 300 && free.height === 150, 'a canvas CSS gives no size keeps its size');
    run(free, clear);
    assert(free.width === 300, 'and keeps it');

    const none = new FakeCanvas();
    none.context = null;
    const error = console.error;
    const errors: Array<unknown> = [];
    console.error = (message: unknown) => { errors.push(message); };
    try {
        run(none, clear);
        run(none, clear);
    } finally {
        console.error = error;
    }
    assert(errors.length === 1, 'a browser without WebGL 2 is told once');
    assert(none.log.length === 0, 'and the calls go nowhere');

    webgls.forget(canvas as unknown as Element);
    run(canvas, calls([1, 0], [31, ...u32(0x4000)]));
    assert(canvas.asked.length === 2, 'a canvas forgotten and back gets its context again');
}
