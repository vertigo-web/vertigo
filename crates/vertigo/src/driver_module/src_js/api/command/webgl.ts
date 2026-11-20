// WebGL 2 on `<canvas>` elements, from `WebGl` in `crates/vertigo/src/webgl.rs`, which defines
// the format of its calls. `webgl.test.ts` plays back a fixture made there. A chunk of its own,
// loaded the first time a page needs it, see `webgl_lazy.ts`: it may import nothing.
//
// Objects are known by ids wasm gives them, unique on the page, and kept here until wasm lets
// them go, the locations of uniforms too. Nothing asks WebGL anything back while drawing, and a
// shader or a program that fails is told in the console.
//
// The bitmap of a canvas is kept as big as the canvas is on the screen, in device pixels, as
// `canvas.ts` keeps it, and it is resized before the calls that come next. Resizing a canvas
// clears it and doesn't change the viewport, so the viewport becomes the whole canvas then. When
// the canvas changes its size, wasm is told the size before anything is resized, so its calls
// drawing again resize the bitmap and draw in one go, and the canvas never shows it cleared.
//
// A context lost (the GPU reset, or given to another tab) is asked back. When it comes back,
// every object is gone: the ids are forgotten, and wasm is told, to make them again.
//
// An image for a texture, from a URL, is loaded meanwhile, and put in the texture when it comes,
// between the calls: wasm is told then, to draw again, or if it didn't load.

const decoder = new TextDecoder("utf-8");

const Op = {
    Context: 1,
    CreateBuffer: 2,
    DeleteBuffer: 3,
    BindBuffer: 4,
    BufferData: 5,
    BufferSubData: 6,
    CreateVertexArray: 7,
    DeleteVertexArray: 8,
    BindVertexArray: 9,
    EnableVertexAttribArray: 10,
    VertexAttribPointer: 11,
    CreateProgram: 12,
    DeleteProgram: 13,
    UseProgram: 14,
    Uniform1f: 15,
    Uniform2f: 16,
    Uniform3f: 17,
    Uniform4f: 18,
    Uniform1i: 19,
    UniformMatrix3f: 20,
    UniformMatrix4f: 21,
    CreateTexture: 22,
    DeleteTexture: 23,
    ActiveTexture: 24,
    BindTexture: 25,
    TextureImage: 26,
    TextureParameter: 27,
    GenerateMipmap: 28,
    ClearColor: 29,
    ClearDepth: 30,
    Clear: 31,
    Enable: 32,
    Disable: 33,
    Viewport: 34,
    DepthFunc: 35,
    BlendFunc: 36,
    CullFace: 37,
    FrontFace: 38,
    DrawArrays: 39,
    DrawElements: 40,
    UniformLocation: 41,
    DeleteUniformLocation: 42,
    VertexAttribDivisor: 43,
    DrawArraysInstanced: 44,
    DrawElementsInstanced: 45,
    TextureImageUrl: 46,
    DepthMask: 47,
    BlendFuncSeparate: 48,
} as const;

// The WebGL constants this file passes itself. Those of the calls come with them.
const GL = {
    FRAGMENT_SHADER: 0x8B30,
    VERTEX_SHADER: 0x8B31,
    COMPILE_STATUS: 0x8B81,
    LINK_STATUS: 0x8B82,
    TEXTURE_2D: 0x0DE1,
    TEXTURE_BINDING_2D: 0x8069,
    TEXTURE0: 0x84C0,
    RGBA: 0x1908,
    RGBA8: 0x8058,
    UNSIGNED_BYTE: 0x1401,
} as const;

// The part of a WebGL 2 context the calls use, so a test can play them back on a recording.
export type Gl = Pick<WebGL2RenderingContext,
    'createBuffer' | 'deleteBuffer' | 'bindBuffer' | 'bufferData' | 'bufferSubData'
    | 'createVertexArray' | 'deleteVertexArray' | 'bindVertexArray' | 'enableVertexAttribArray'
    | 'vertexAttribPointer' | 'createShader' | 'shaderSource' | 'compileShader' | 'getShaderParameter'
    | 'getShaderInfoLog' | 'deleteShader' | 'createProgram' | 'attachShader' | 'linkProgram'
    | 'getProgramParameter' | 'getProgramInfoLog' | 'deleteProgram' | 'useProgram'
    | 'getUniformLocation' | 'uniform1f' | 'uniform2f' | 'uniform3f' | 'uniform4f' | 'uniform1i'
    | 'uniformMatrix3fv' | 'uniformMatrix4fv' | 'createTexture' | 'deleteTexture' | 'activeTexture'
    | 'bindTexture' | 'texImage2D' | 'texParameteri' | 'generateMipmap' | 'clearColor' | 'clearDepth'
    | 'clear' | 'enable' | 'disable' | 'viewport' | 'depthFunc' | 'blendFunc' | 'cullFace'
    | 'frontFace' | 'drawArrays' | 'drawElements' | 'vertexAttribDivisor' | 'drawArraysInstanced'
    | 'drawElementsInstanced' | 'depthMask' | 'blendFuncSeparate' | 'getParameter'>;

// Asks for the image at `url` for the texture of the id `texture`, see `Op.TextureImageUrl`.
export type Load = (texture: number, url: string, mipmap: boolean) => void;

// The objects of a context by their ids.
export class Objects {
    public readonly buffers: Map<number, WebGLBuffer | null> = new Map();
    public readonly vertexArrays: Map<number, WebGLVertexArrayObject | null> = new Map();
    public readonly programs: Map<number, WebGLProgram | null> = new Map();
    public readonly textures: Map<number, WebGLTexture | null> = new Map();
    // `null` for a uniform its program hasn't.
    public readonly locations: Map<number, WebGLUniformLocation | null> = new Map();
}

// A matrix for a uniform, filled anew for each: WebGL copies it.
const matrix3 = new Float32Array(9);
const matrix4 = new Float32Array(16);

// The attributes of `getContext`, from the flags of `Op.Context`, see `WebGlOptions`.
export const attributes = (flags: number): WebGLContextAttributes => ({
    alpha: (flags & 1) !== 0,
    antialias: (flags & 2) !== 0,
    depth: (flags & 4) !== 0,
    stencil: (flags & 8) !== 0,
    premultipliedAlpha: (flags & 16) !== 0,
    preserveDrawingBuffer: (flags & 32) !== 0,
});

// Whether `value`, which WebGL answered for a binding, is a texture rather than `null`.
const isTexture = (value: unknown): value is WebGLTexture => typeof value === 'object' && value !== null;

// The object of `id` in `map`, `null` for 0 and for one let go.
const objectOf = <T>(map: Map<number, T | null>, id: number): T | null =>
    id === 0 ? null : map.get(id) ?? null;

const compile = (gl: Gl, type: number, source: string): WebGLShader | null => {
    const shader = gl.createShader(type);
    if (shader === null) {
        return null;
    }

    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (gl.getShaderParameter(shader, GL.COMPILE_STATUS) !== true) {
        const kind = type === GL.VERTEX_SHADER ? 'vertex' : 'fragment';
        console.error(`webgl: the ${kind} shader doesn't compile:\n${gl.getShaderInfoLog(shader) ?? ''}`);
    }
    return shader;
};

// A program of the two shaders, linked. The shaders go with it, as nothing else uses them.
const link = (gl: Gl, vertex: string, fragment: string): WebGLProgram | null => {
    const program = gl.createProgram();
    const shaders = [compile(gl, GL.VERTEX_SHADER, vertex), compile(gl, GL.FRAGMENT_SHADER, fragment)];

    for (const shader of shaders) {
        if (shader !== null) {
            gl.attachShader(program, shader);
        }
    }
    gl.linkProgram(program);
    if (gl.getProgramParameter(program, GL.LINK_STATUS) !== true) {
        console.error(`webgl: the program doesn't link:\n${gl.getProgramInfoLog(program) ?? ''}`);
    }
    for (const shader of shaders) {
        gl.deleteShader(shader);
    }

    return program;
};

// Plays `bytes` back on `gl`, with the objects made so far in `objects`. The images for textures
// are asked of `load`.
export const play = (gl: Gl, objects: Objects, bytes: Uint8Array, load?: Load): void => {
    const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    let at = 0;

    const u8 = (): number => {
        const value = data.getUint8(at);
        at += 1;
        return value;
    };

    const u32 = (): number => {
        const value = data.getUint32(at, true);
        at += 4;
        return value;
    };

    const i32 = (): number => {
        const value = data.getInt32(at, true);
        at += 4;
        return value;
    };

    const f32 = (): number => {
        const value = data.getFloat32(at, true);
        at += 4;
        return value;
    };

    const filled = (matrix: Float32Array): Float32Array => {
        for (let index = 0; index < matrix.length; index++) {
            matrix[index] = f32();
        }
        return matrix;
    };

    // A view of the bytes, good while this call runs, as WebGL copies what it takes.
    const block = (): Uint8Array => {
        const length = u32();
        const value = bytes.subarray(at, at + length);
        at += length;
        return value;
    };

    const location = (): WebGLUniformLocation | null => objects.locations.get(u32()) ?? null;

    while (at < bytes.length) {
        const op = u8();

        switch (op) {
            case Op.Context:
                // Taken by `WebGls.run` when it made the context: the flags and the callback.
                at += 9;
                break;
            case Op.CreateBuffer:
                objects.buffers.set(u32(), gl.createBuffer());
                break;
            case Op.DeleteBuffer: {
                const id = u32();
                gl.deleteBuffer(objectOf(objects.buffers, id));
                objects.buffers.delete(id);
                break;
            }
            case Op.BindBuffer: {
                const target = u32();
                gl.bindBuffer(target, objectOf(objects.buffers, u32()));
                break;
            }
            case Op.BufferData: {
                const target = u32();
                const usage = u32();
                gl.bufferData(target, block(), usage);
                break;
            }
            case Op.BufferSubData: {
                const target = u32();
                const offset = u32();
                gl.bufferSubData(target, offset, block());
                break;
            }
            case Op.CreateVertexArray:
                objects.vertexArrays.set(u32(), gl.createVertexArray());
                break;
            case Op.DeleteVertexArray: {
                const id = u32();
                gl.deleteVertexArray(objectOf(objects.vertexArrays, id));
                objects.vertexArrays.delete(id);
                break;
            }
            case Op.BindVertexArray:
                gl.bindVertexArray(objectOf(objects.vertexArrays, u32()));
                break;
            case Op.EnableVertexAttribArray:
                gl.enableVertexAttribArray(u32());
                break;
            case Op.VertexAttribPointer: {
                const index = u32();
                const size = u8();
                const type = u32();
                const normalized = u8() !== 0;
                const stride = u32();
                gl.vertexAttribPointer(index, size, type, normalized, stride, u32());
                break;
            }
            case Op.CreateProgram: {
                const id = u32();
                const vertex = decoder.decode(block());
                const fragment = decoder.decode(block());
                objects.programs.set(id, link(gl, vertex, fragment));
                break;
            }
            case Op.DeleteProgram: {
                const id = u32();
                gl.deleteProgram(objectOf(objects.programs, id));
                objects.programs.delete(id);
                break;
            }
            case Op.UseProgram:
                gl.useProgram(objectOf(objects.programs, u32()));
                break;
            case Op.Uniform1f:
                gl.uniform1f(location(), f32());
                break;
            case Op.Uniform2f:
                gl.uniform2f(location(), f32(), f32());
                break;
            case Op.Uniform3f:
                gl.uniform3f(location(), f32(), f32(), f32());
                break;
            case Op.Uniform4f:
                gl.uniform4f(location(), f32(), f32(), f32(), f32());
                break;
            case Op.Uniform1i:
                gl.uniform1i(location(), i32());
                break;
            case Op.UniformMatrix3f:
                gl.uniformMatrix3fv(location(), false, filled(matrix3));
                break;
            case Op.UniformMatrix4f:
                gl.uniformMatrix4fv(location(), false, filled(matrix4));
                break;
            case Op.CreateTexture:
                objects.textures.set(u32(), gl.createTexture());
                break;
            case Op.DeleteTexture: {
                const id = u32();
                gl.deleteTexture(objectOf(objects.textures, id));
                objects.textures.delete(id);
                break;
            }
            case Op.ActiveTexture:
                gl.activeTexture(GL.TEXTURE0 + u32());
                break;
            case Op.BindTexture:
                gl.bindTexture(GL.TEXTURE_2D, objectOf(objects.textures, u32()));
                break;
            case Op.TextureImage: {
                const width = u32();
                const height = u32();
                gl.texImage2D(GL.TEXTURE_2D, 0, GL.RGBA8, width, height, 0, GL.RGBA, GL.UNSIGNED_BYTE, block());
                break;
            }
            case Op.TextureParameter: {
                const name = u32();
                gl.texParameteri(GL.TEXTURE_2D, name, u32());
                break;
            }
            case Op.GenerateMipmap:
                gl.generateMipmap(GL.TEXTURE_2D);
                break;
            case Op.ClearColor:
                gl.clearColor(f32(), f32(), f32(), f32());
                break;
            case Op.ClearDepth:
                gl.clearDepth(f32());
                break;
            case Op.Clear:
                gl.clear(u32());
                break;
            case Op.Enable:
                gl.enable(u32());
                break;
            case Op.Disable:
                gl.disable(u32());
                break;
            case Op.Viewport:
                gl.viewport(i32(), i32(), i32(), i32());
                break;
            case Op.DepthFunc:
                gl.depthFunc(u32());
                break;
            case Op.BlendFunc:
                gl.blendFunc(u32(), u32());
                break;
            case Op.CullFace:
                gl.cullFace(u32());
                break;
            case Op.FrontFace:
                gl.frontFace(u32());
                break;
            case Op.DrawArrays:
                gl.drawArrays(u32(), u32(), u32());
                break;
            case Op.DrawElements:
                gl.drawElements(u32(), u32(), u32(), u32());
                break;
            case Op.UniformLocation: {
                const id = u32();
                const program = objectOf(objects.programs, u32());
                const length = data.getUint16(at, true);
                const name = decoder.decode(bytes.subarray(at + 2, at + 2 + length));
                at += 2 + length;
                objects.locations.set(id, program === null ? null : gl.getUniformLocation(program, name));
                break;
            }
            case Op.DeleteUniformLocation:
                objects.locations.delete(u32());
                break;
            case Op.VertexAttribDivisor:
                gl.vertexAttribDivisor(u32(), u32());
                break;
            case Op.DrawArraysInstanced:
                gl.drawArraysInstanced(u32(), u32(), u32(), u32());
                break;
            case Op.DrawElementsInstanced:
                gl.drawElementsInstanced(u32(), u32(), u32(), u32(), u32());
                break;
            case Op.TextureImageUrl: {
                const texture = u32();
                const mipmap = u8() !== 0;
                const url = decoder.decode(block());
                load?.(texture, url, mipmap);
                break;
            }
            case Op.DepthMask:
                gl.depthMask(u8() !== 0);
                break;
            case Op.BlendFuncSeparate:
                gl.blendFuncSeparate(u32(), u32(), u32(), u32());
                break;
            default:
                throw new Error(`webgl: unknown operation ${op}`);
        }
    }
};

type Size = { width: number, height: number };

// Tells wasm of a canvas, through the callback of its context: `[0, width, height]` when it
// changes its size, `[1]` when its context is given back, `[2, texture, width, height]` when an
// image is in a texture, `[3, texture]` when it didn't load. See `CanvasEvent` in webgl.rs.
export type Tell = (callback: number, event: Array<number>) => void;

// A canvas's context, with its objects and what wasm was told of it.
class Context {
    public objects: Objects = new Objects();
    public told: Size | null = null;
    // The URL of the image each texture waits for, by its id: only the last asked for goes in.
    public readonly loading: Map<number, string> = new Map();

    constructor(
        public readonly gl: WebGL2RenderingContext,
        public readonly callback: number,
        public readonly listeners: Array<[string, (event: Event) => void]>,
    ) {}
}

export class WebGls {
    // `null` for a canvas without WebGL 2 in the browser.
    private readonly contexts: WeakMap<HTMLCanvasElement, Context | null> = new WeakMap();
    // The size in device pixels each canvas was seen at last, see `fit`.
    private readonly sizes: WeakMap<HTMLCanvasElement, Size> = new WeakMap();
    // Canvases as big as their bitmaps, which CSS gives no size, see `fit`.
    private readonly unscaled: WeakSet<HTMLCanvasElement> = new WeakSet();
    private observer: ResizeObserver | null = null;

    public constructor(private readonly tell: Tell) {}

    // `bytes` is good only for the call it came in (see `DriverDom.update`): it is played now.
    public run(node: Element, bytes: Uint8Array) {
        if (!(node instanceof HTMLCanvasElement)) {
            throw new Error('webgl: the element is not a canvas');
        }

        let context = this.contexts.get(node);

        if (context === undefined) {
            context = this.make(node, bytes);
            this.contexts.set(node, context);
            this.observe(node);
        }

        if (context !== null) {
            this.fit(node, context.gl);
            const known = context;
            play(context.gl, context.objects, bytes, (texture, url, mipmap) => {
                this.load(known, texture, url, mipmap);
            });
        }
    }

    public forget(node: Element | Comment | Text) {
        if (node instanceof HTMLCanvasElement && this.contexts.has(node)) {
            for (const [type, listener] of this.contexts.get(node)?.listeners ?? []) {
                node.removeEventListener(type, listener);
            }
            this.observer?.unobserve(node);
            this.contexts.delete(node);
            this.sizes.delete(node);
            this.unscaled.delete(node);
        }
    }

    // The context, from the first calls of it: they begin with its attributes and its callback.
    private make(canvas: HTMLCanvasElement, bytes: Uint8Array): Context | null {
        const first = bytes[0] === Op.Context && bytes.length >= 10;
        const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const flags = first ? data.getUint8(1) : 0;
        const callback = first ? data.getUint32(2, true) + data.getUint32(6, true) * 2 ** 32 : 0;

        const gl = canvas.getContext('webgl2', attributes(flags));
        if (gl === null) {
            console.error('webgl: the browser has no WebGL 2, or the canvas has another context');
            return null;
        }

        const lost = (event: Event) => {
            // Asks for it back.
            event.preventDefault();
        };
        const restored = () => {
            const context = this.contexts.get(canvas);
            if (context) {
                context.objects = new Objects();
                this.told(context, [1]);
            }
        };
        canvas.addEventListener('webglcontextlost', lost);
        canvas.addEventListener('webglcontextrestored', restored);

        return new Context(gl, callback, [['webglcontextlost', lost], ['webglcontextrestored', restored]]);
    }

    // Puts the image at `url` in the texture when it comes, unless the texture was let go
    // meanwhile, or asked for another image, or the context was lost.
    private load(context: Context, texture: number, url: string, mipmap: boolean) {
        const objects = context.objects;
        context.loading.set(texture, url);
        const current = () => context.objects === objects && context.loading.get(texture) === url;

        const image = new Image();
        image.crossOrigin = 'anonymous';
        image.onload = () => {
            if (!current()) {
                return;
            }
            context.loading.delete(texture);
            const target = objects.textures.get(texture);
            if (!target) {
                return;
            }
            const gl = context.gl;
            const bound: unknown = gl.getParameter(GL.TEXTURE_BINDING_2D);
            gl.bindTexture(GL.TEXTURE_2D, target);
            gl.texImage2D(GL.TEXTURE_2D, 0, GL.RGBA8, GL.RGBA, GL.UNSIGNED_BYTE, image);
            if (mipmap) {
                gl.generateMipmap(GL.TEXTURE_2D);
            }
            gl.bindTexture(GL.TEXTURE_2D, isTexture(bound) ? bound : null);
            this.told(context, [2, texture, image.naturalWidth, image.naturalHeight]);
        };
        image.onerror = () => {
            if (!current()) {
                return;
            }
            context.loading.delete(texture);
            console.error(`webgl: the image ${url} didn't load`);
            this.told(context, [3, texture]);
        };
        image.src = url;
    }

    private told(context: Context, event: Array<number>) {
        if (context.callback === 0) {
            return;
        }
        try {
            this.tell(context.callback, event);
        } catch (error) {
            console.error(error);
        }
    }

    private observe(canvas: HTMLCanvasElement) {
        if (typeof ResizeObserver === 'undefined') {
            return;
        }

        if (this.observer === null) {
            this.observer = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    if (entry.target instanceof HTMLCanvasElement) {
                        this.resized(entry.target, entry.contentRect);
                    }
                }
            });
        }

        try {
            this.observer.observe(canvas, { box: 'device-pixel-content-box' });
        } catch {
            this.observer.observe(canvas);
        }
    }

    // The canvas's new size, kept for the calls that come next to resize the bitmap to, and told:
    // calls drawing again come at once then. A canvas as big as its bitmap keeps its size.
    private resized(canvas: HTMLCanvasElement, box: { width: number, height: number }) {
        const ratio = window.devicePixelRatio || 1;
        const size = {
            width: Math.round(box.width * ratio),
            height: Math.round(box.height * ratio),
        };
        this.sizes.set(canvas, size);

        const context = this.contexts.get(canvas);
        if (!context) {
            return;
        }

        const told = this.unscaled.has(canvas) ? { width: canvas.width, height: canvas.height } : size;
        if (told.width > 0 && told.height > 0
            && (context.told?.width !== told.width || context.told.height !== told.height)) {
            context.told = told;
            this.told(context, [0, told.width, told.height]);
        }
    }

    // Makes the bitmap as big as the canvas is on the screen, before the calls that draw on it.
    // A canvas CSS gives no size is as big as its bitmap, which would grow with each resize at a
    // ratio above 1: it keeps the size it has, a pixel to a CSS pixel.
    private fit(canvas: HTMLCanvasElement, gl: WebGL2RenderingContext) {
        if (this.unscaled.has(canvas)) {
            return;
        }

        const ratio = window.devicePixelRatio || 1;
        const size = this.sizes.get(canvas) ?? (canvas.isConnected ? {
            width: Math.round(canvas.clientWidth * ratio),
            height: Math.round(canvas.clientHeight * ratio),
        } : null);

        if (size === null || size.width <= 0 || size.height <= 0
            || (canvas.width === size.width && canvas.height === size.height)) {
            return;
        }

        const [width, height] = [canvas.width, canvas.height];
        const [cssWidth, cssHeight] = [canvas.clientWidth, canvas.clientHeight];
        canvas.width = size.width;
        canvas.height = size.height;

        if (ratio !== 1
            && canvas.clientWidth - cssWidth === size.width - width
            && canvas.clientHeight - cssHeight === size.height - height) {
            canvas.width = width;
            canvas.height = height;
            this.unscaled.add(canvas);
        }

        gl.viewport(0, 0, canvas.width, canvas.height);
    }
}
