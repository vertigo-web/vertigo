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
        getParameter: () => ({ name: 'bound' }),
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
    1, 63, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 2, 1, 0, 0, 0, 4, 146, 136, 0, 0, 1, 0, 0, 0, 5,
    146, 136, 0, 0, 228, 136, 0, 0, 8, 0, 0, 0, 0, 0, 192, 63, 0, 0, 0, 192, 6, 147, 136, 0, 0, 2,
    0, 0, 0, 4, 0, 0, 0, 7, 0, 8, 0, 7, 2, 0, 0, 0, 9, 2, 0, 0, 0, 10, 1, 0, 0, 0, 11, 1, 0, 0, 0,
    3, 6, 20, 0, 0, 1, 12, 0, 0, 0, 4, 0, 0, 0, 12, 3, 0, 0, 0, 2, 0, 0, 0, 118, 115, 3, 0, 0, 0,
    102, 197, 155, 14, 3, 0, 0, 0, 41, 4, 0, 0, 0, 3, 0, 0, 0, 1, 0, 117, 15, 4, 0, 0, 0, 0, 0, 0,
    63, 16, 4, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 17, 4, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0,
    0, 64, 64, 18, 4, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 19, 4, 0, 0,
    0, 255, 255, 255, 255, 20, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0,
    0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0, 224, 64, 0, 0, 0, 65, 21, 4, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64, 0, 0, 160, 64, 0, 0, 192, 64, 0, 0,
    224, 64, 0, 0, 0, 65, 0, 0, 16, 65, 0, 0, 32, 65, 0, 0, 48, 65, 0, 0, 64, 65, 0, 0, 80, 65, 0,
    0, 96, 65, 0, 0, 112, 65, 22, 5, 0, 0, 0, 24, 2, 0, 0, 0, 25, 5, 0, 0, 0, 26, 1, 0, 0, 0, 1, 0,
    0, 0, 4, 0, 0, 0, 255, 0, 128, 255, 27, 1, 40, 0, 0, 0, 38, 0, 0, 27, 0, 40, 0, 0, 1, 38, 0, 0,
    27, 2, 40, 0, 0, 1, 41, 0, 0, 27, 3, 40, 0, 0, 47, 129, 0, 0, 28, 29, 0, 0, 0, 0, 0, 0, 128, 62,
    0, 0, 0, 63, 0, 0, 128, 63, 30, 0, 0, 128, 63, 31, 0, 65, 0, 0, 32, 113, 11, 0, 0, 33, 226, 11,
    0, 0, 34, 0, 0, 0, 0, 255, 255, 255, 255, 128, 2, 0, 0, 224, 1, 0, 0, 35, 3, 2, 0, 0, 36, 2, 3,
    0, 0, 3, 3, 0, 0, 37, 5, 4, 0, 0, 38, 0, 9, 0, 0, 39, 4, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 40, 1,
    0, 0, 0, 6, 0, 0, 0, 3, 20, 0, 0, 2, 0, 0, 0, 43, 3, 0, 0, 0, 1, 0, 0, 0, 44, 4, 0, 0, 0, 0, 0,
    0, 0, 3, 0, 0, 0, 10, 0, 0, 0, 45, 4, 0, 0, 0, 36, 0, 0, 0, 3, 20, 0, 0, 0, 0, 0, 0, 144, 1, 0,
    0, 46, 5, 0, 0, 0, 1, 7, 0, 0, 0, 47, 197, 130, 46, 112, 110, 103, 47, 0, 48, 2, 3, 0, 0, 3, 3,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 49, 65, 146, 0, 0, 1, 0, 0, 0, 50, 0, 0, 128, 62, 4, 146, 136, 0,
    0, 0, 0, 0, 0, 3, 1, 0, 0, 0, 8, 2, 0, 0, 0, 13, 3, 0, 0, 0, 23, 5, 0, 0, 0, 42, 4, 0, 0, 0,
]);

console.log("\n--- Test webgl: the cross-language fixture ---");
{
    const log: Array<string> = [];
    play(recording(log), new Objects(), FIXTURE, (texture, url, mipmap, unpack) => {
        log.push(`load(${texture},${url},${String(mipmap)},${JSON.stringify([...unpack])})`);
    }, (scale) => log.push(`rescale(${scale})`));

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
        'getUniformLocation(program3,u)',
        'uniform1f(@u,0.5)',
        'uniform2f(@u,1,2)',
        'uniform3f(@u,1,2,3)',
        'uniform4f(@u,1,2,3,4)',
        'uniform1i(@u,-1)',
        'uniformMatrix3fv(@u,false,[0,1,2,3,4,5,6,7,8])',
        'uniformMatrix4fv(@u,false,[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15])',
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
        'vertexAttribDivisor(3,1)',
        'drawArraysInstanced(4,0,3,10)',
        'drawElementsInstanced(4,36,5123,0,400)',
        'load(5,/ł.png,true,[])',
        'depthMask(false)',
        'blendFuncSeparate(770,771,1,0)',
        'pixelStorei(37441,1)',
        'rescale(0.25)',
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
    play(gl, objects, calls([41, ...u32(2), ...u32(1), ...name('a')]));
    same(log, ['getUniformLocation(program1,a)'], "a uniform's location is found when it is made");

    log.length = 0;
    play(gl, objects, calls([14, ...u32(1)], [15, ...u32(2), 0, 0, 128, 63], [15, ...u32(2), 0, 0, 0, 64]));
    same(log, ['useProgram(program1)', 'uniform1f(@a,1)', 'uniform1f(@a,2)'],
        'and not again when the uniform is set');

    log.length = 0;
    play(gl, objects, calls([42, ...u32(2)], [15, ...u32(2), 0, 0, 128, 63]));
    same(log, ['uniform1f(null,1)'], 'a location let go sets nothing, which WebGL ignores');

    log.length = 0;
    play(gl, objects, calls([13, ...u32(1)], [41, ...u32(3), ...u32(1), ...name('a')], [15, ...u32(3), 0, 0, 128, 63]));
    same(log, ['deleteProgram(program1)', 'uniform1f(null,1)'], 'nor one of a program let go');

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
        public readonly listeners: Map<string, (event: Event) => void> = new Map();
        get clientWidth() { return this.css?.width ?? this.width; }
        get clientHeight() { return this.css?.height ?? this.height; }
        getContext(type: string, options: WebGLContextAttributes) {
            this.asked.push(`${type} preserveDrawingBuffer=${String(options.preserveDrawingBuffer)}`);
            return this.context;
        }
        addEventListener(type: string, listener: (event: Event) => void) { this.listeners.set(type, listener); }
        removeEventListener(type: string) { this.listeners.delete(type); }
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

    // What the canvases told wasm, by callback.
    const told: Array<string> = [];
    const webgls = new WebGls((callback, event) => { told.push(`${callback}: ${event.join(',')}`); });
    const run = (canvas: FakeCanvas, bytes: Uint8Array) => webgls.run(canvas as unknown as Element, bytes);
    const resized = (canvas: FakeCanvas, width: number, height: number) =>
        watchers[0]?.([{ target: canvas, contentRect: { width, height } }]);
    const f32 = (value: number) => Array.from(new Uint8Array(new Float32Array([value]).buffer));
    // The first call of a context: its attributes, its callback and the scale of its bitmap.
    const context = (flags: number, callback: number, scale = 1) =>
        [1, flags, ...u32(callback), ...u32(0), ...f32(scale)];
    const clear = calls([31, ...u32(0x4000)]);

    const canvas = new FakeCanvas();
    run(canvas, calls(context(32, 7), [31, ...u32(0x4000)]));
    same(canvas.asked, ['webgl2 preserveDrawingBuffer=true'], 'the first calls make the context, with its attributes');
    same(canvas.log, ['viewport(0,0,320,200)', 'clear(16384)'],
        'on a bitmap as big as the canvas in device pixels, all of it drawn on');
    assert(canvas.width === 320 && canvas.height === 200, 'which is 320 x 200');

    canvas.log.length = 0;
    run(canvas, clear);
    same(canvas.asked, ['webgl2 preserveDrawingBuffer=true'], 'the context is made once');
    same(canvas.log, ['clear(16384)'], 'and the bitmap is resized only when the canvas is');

    resized(canvas, 160, 100);
    same(told, ['7: 0,320,200'], 'wasm is told the size once the canvas is seen');
    resized(canvas, 160, 100);
    same(told, ['7: 0,320,200'], 'and not again for the same size');

    resized(canvas, 200, 100);
    same(told, ['7: 0,320,200', '7: 0,400,200'], 'a new size is told before the bitmap changes');
    assert(canvas.width === 320, 'which keeps its picture until the next calls, drawing it again');
    canvas.log.length = 0;
    run(canvas, clear);
    same(canvas.log, ['viewport(0,0,400,200)', 'clear(16384)'], 'which get a bitmap of the new size');

    // Half the pixels each way: a bitmap of 160 x 100 for 320 x 200 device pixels.
    const half = new FakeCanvas();
    run(half, calls(context(0, 10, 0.5), [31, ...u32(0x4000)]));
    same(half.log, ['viewport(0,0,160,100)', 'clear(16384)'], 'a scaled bitmap is drawn on whole');
    assert(half.width === 160 && half.height === 100, 'which is the canvas in device pixels times the scale');
    resized(half, 160, 100);
    same(told.slice(2), ['10: 0,160,100'], 'and its size is told so');
    told.splice(2, 1);

    // The whole canvas again, on the context it has.
    run(half, calls([50, ...f32(1)]));
    same(told.slice(2), ['10: 0,320,200'], 'a new scale is told as a new size');
    half.log.length = 0;
    run(half, clear);
    same(half.log, ['viewport(0,0,320,200)', 'clear(16384)'], 'which the calls after draw on');
    same(half.asked, ['webgl2 preserveDrawingBuffer=false'], 'with the same context');
    told.splice(2, 1);

    // A canvas CSS gives no size is as big as its bitmap.
    const free = new FakeCanvas();
    free.css = null;
    run(free, calls(context(0, 8)));
    assert(free.width === 300 && free.height === 150, 'a canvas CSS gives no size keeps its size');
    resized(free, 300, 150);
    same(told.slice(2), ['8: 0,300,150'], 'and is told so');
    run(free, clear);
    assert(free.width === 300, 'and keeps it');

    // Lost, and given back.
    let prevented = false;
    canvas.listeners.get('webglcontextlost')?.({ preventDefault: () => { prevented = true; } } as Event);
    assert(prevented, 'a context lost is asked back');
    run(canvas, calls([2, ...u32(1)]));
    canvas.listeners.get('webglcontextrestored')?.({} as Event);
    same(told.slice(3), ['7: 1'], 'and wasm is told when it comes back');
    canvas.log.length = 0;
    run(canvas, calls([4, ...u32(34962), ...u32(1)]));
    same(canvas.log, ['bindBuffer(34962,null)'], 'its objects made before are gone');

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

    const unavailable = new FakeCanvas();
    unavailable.context = null;
    console.error = () => {};
    try {
        run(unavailable, calls(context(0, 12)));
    } finally {
        console.error = error;
    }
    same(told.slice(4), ['12: 4'], 'and wasm is told, through the callback of the context');
    told.splice(4, 1);

    // Images for textures.
    const images: Array<FakeImage> = [];
    class FakeImage {
        public crossOrigin = '';
        public src = '';
        public naturalWidth = 64;
        public naturalHeight = 32;
        public onload: () => void = () => {};
        public onerror: () => void = () => {};
        constructor() { images.push(this); }
    }
    global['Image'] = FakeImage;
    const texture = (id: number) => [22, ...u32(id)];
    const image = (id: number, url: string, mipmap: boolean) => [46, ...u32(id), mipmap ? 1 : 0, ...source(url)];

    run(canvas, calls(texture(10), image(10, '/a.png', true), texture(11), image(11, '/b.png', false)));
    same(images.map((image) => `${image.src} ${image.crossOrigin}`), ['/a.png anonymous', '/b.png anonymous'],
        'an image is loaded for its texture, from any origin allowing it');
    canvas.log.length = 0;
    images[0]?.onload();
    same(canvas.log, [
        'getParameter(32873)',
        'bindTexture(3553,texture2)',
        'texImage2D(3553,0,32856,6408,5121,[object Object])',
        'generateMipmap(3553)',
        'bindTexture(3553,bound)',
    ], 'and put in it when it comes, the texture bound before bound again');
    same(told.slice(4), ['7: 2,10,64,32'], 'and wasm is told, with its size');

    const error2 = console.error;
    console.error = () => {};
    try {
        images[1]?.onerror();
    } finally {
        console.error = error2;
    }
    same(told.slice(5), ['7: 3,11'], 'as of an image that did not load');

    run(canvas, calls(image(10, '/c.png', false), image(10, '/d.png', false)));
    canvas.log.length = 0;
    images[2]?.onload();
    images[3]?.onload();
    same(told.slice(6), ['7: 2,10,64,32'], 'of two images for a texture, the last asked for goes in');
    assert(canvas.log.filter((call) => call.startsWith('texImage2D')).length === 1, 'and only it');

    // Premultiplied as asked for then, and not after.
    const premultiply = (on: boolean) => [49, ...u32(0x9241), ...u32(on ? 1 : 0)];
    run(canvas, calls(premultiply(true), image(10, '/f.png', true), premultiply(false), image(11, '/g.png', false)));
    canvas.log.length = 0;
    images[4]?.onload();
    images[5]?.onload();
    same(canvas.log, [
        'getParameter(32873)',
        'bindTexture(3553,texture2)',
        'pixelStorei(37441,1)',
        'texImage2D(3553,0,32856,6408,5121,[object Object])',
        'generateMipmap(3553)',
        'pixelStorei(37441,0)',
        'bindTexture(3553,bound)',
        'getParameter(32873)',
        'bindTexture(3553,texture3)',
        'texImage2D(3553,0,32856,6408,5121,[object Object])',
        'bindTexture(3553,bound)',
    ], 'an image is read with the pixel storage modes there were when it was asked for');
    told.splice(7, 2);

    run(canvas, calls(image(11, '/e.png', false)));
    canvas.listeners.get('webglcontextrestored')?.({} as Event);
    images[6]?.onload();
    same(told.slice(7), ['7: 1'], 'an image of a context lost meanwhile goes nowhere');

    run(canvas, calls(texture(12), image(12, '/f.png', false)));
    webgls.forget(canvas as unknown as Element);
    assert(canvas.listeners.size === 0, 'a canvas forgotten is listened to no more');
    assert(images.length === 8, 'the image asked for before');
    images[7]?.onload();
    same(told.slice(8), [], 'and an image coming for it after goes nowhere');
    run(canvas, calls(context(0, 9), [31, ...u32(0x4000)]));
    assert(canvas.asked.length === 2, 'and gets its context again when back');
}
