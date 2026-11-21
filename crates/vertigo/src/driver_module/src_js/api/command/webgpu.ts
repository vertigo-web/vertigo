// WebGPU on `<canvas>` elements, from `WebGpu` in `crates/vertigo/src/webgpu.rs`, which defines
// the format of its calls. `webgpu.test.ts` plays back a fixture made there. In the chunk of
// `gpu.ts`, with WebGL, loaded the first time a page needs either, see `gpu_lazy.ts`.
//
// Each canvas has a device of its own, asked for with its first calls, which wait for it. The
// canvas gets its context only once there is a device, so that a canvas the browser has no WebGPU
// for is still free for WebGL. Objects are known by ids wasm gives them, and kept here until wasm
// lets them go. Nothing asks WebGPU anything back while drawing, and what WebGPU finds wrong is
// told in the console.
//
// The passes of a batch are encoded into one command buffer, which goes to the queue at the end
// of the batch, or before a write, so that the write comes after the passes made before it. The
// buffers and textures let go during a batch are destroyed after it, as its passes may use them.
//
// The bitmap of a canvas is kept as big as the canvas is on the screen, see `canvas_sizes.ts`, and
// so is a depth texture of it, made again when the bitmap is resized.
//
// A device lost is asked for again. When one comes, every object is gone: the ids are forgotten,
// and wasm is told, to make them again. Wasm is told `[0, width, height]` when the canvas changes
// its size, `[1]` when the context has a new device, `[2]` when there is no WebGPU for it, see
// `GpuEvent` in webgpu.rs.

import { CanvasSizes, Size, Tell } from "./canvas_sizes";

const decoder = new TextDecoder("utf-8");

const Op = {
    Context: 1,
    CreateBuffer: 2,
    DestroyBuffer: 3,
    WriteBuffer: 4,
    CreateTexture: 5,
    DestroyTexture: 6,
    WriteTexture: 7,
    CreateSampler: 8,
    DeleteSampler: 9,
    CreateShaderModule: 10,
    DeleteShaderModule: 11,
    CreateRenderPipeline: 12,
    CreateComputePipeline: 13,
    DeletePipeline: 14,
    CreateBindGroup: 15,
    DeleteBindGroup: 16,
    BeginRenderPass: 17,
    BeginComputePass: 18,
    EndPass: 19,
    SetPipeline: 20,
    SetBindGroup: 21,
    SetVertexBuffer: 22,
    SetIndexBuffer: 23,
    Draw: 24,
    DrawIndexed: 25,
    DispatchWorkgroups: 26,
    SetViewport: 27,
    SetScissorRect: 28,
} as const;

// WebGPU's strings, at the places of the values of the enums in webgpu.rs. `null` is the format of
// the canvas.
const TEXTURE_FORMATS: ReadonlyArray<GPUTextureFormat | null> = [
    null, 'rgba8unorm', 'rgba8unorm-srgb', 'bgra8unorm', 'bgra8unorm-srgb', 'r8unorm', 'rg8unorm',
    'r16float', 'rg16float', 'rgba16float', 'r32float', 'rg32float', 'rgba32float', 'r32uint',
    'depth16unorm', 'depth24plus', 'depth24plus-stencil8', 'depth32float',
];
const VERTEX_FORMATS: ReadonlyArray<GPUVertexFormat> = [
    'float32', 'float32x2', 'float32x3', 'float32x4', 'uint32', 'uint32x2', 'uint32x3', 'uint32x4',
    'sint32', 'sint32x2', 'sint32x3', 'sint32x4', 'unorm8x4', 'uint8x4', 'snorm8x4', 'float16x2',
    'float16x4', 'uint16x2', 'unorm16x2',
];
const STEP_MODES: ReadonlyArray<GPUVertexStepMode> = ['vertex', 'instance'];
const TOPOLOGIES: ReadonlyArray<GPUPrimitiveTopology> = [
    'point-list', 'line-list', 'line-strip', 'triangle-list', 'triangle-strip',
];
const INDEX_FORMATS: ReadonlyArray<GPUIndexFormat> = ['uint16', 'uint32'];
const FRONT_FACES: ReadonlyArray<GPUFrontFace> = ['ccw', 'cw'];
const CULL_MODES: ReadonlyArray<GPUCullMode> = ['none', 'front', 'back'];
const COMPARE_FUNCTIONS: ReadonlyArray<GPUCompareFunction> = [
    'never', 'less', 'equal', 'less-equal', 'greater', 'not-equal', 'greater-equal', 'always',
];
const BLEND_FACTORS: ReadonlyArray<GPUBlendFactor> = [
    'zero', 'one', 'src', 'one-minus-src', 'src-alpha', 'one-minus-src-alpha', 'dst',
    'one-minus-dst', 'dst-alpha', 'one-minus-dst-alpha', 'src-alpha-saturated', 'constant',
    'one-minus-constant',
];
const BLEND_OPERATIONS: ReadonlyArray<GPUBlendOperation> = [
    'add', 'subtract', 'reverse-subtract', 'min', 'max',
];
const ADDRESS_MODES: ReadonlyArray<GPUAddressMode> = ['clamp-to-edge', 'repeat', 'mirror-repeat'];
const FILTER_MODES: ReadonlyArray<GPUFilterMode> = ['nearest', 'linear'];
const ALPHA_MODES: ReadonlyArray<GPUCanvasAlphaMode> = ['opaque', 'premultiplied'];
const POWER_PREFERENCES: ReadonlyArray<GPUPowerPreference | undefined> = [
    undefined, 'low-power', 'high-performance',
];

// `GPUTextureUsage.RENDER_ATTACHMENT`, which isn't there outside of a browser.
const RENDER_ATTACHMENT = 0x10;

// The value at `index` of `values`, see the lists above.
const pick = <T>(values: ReadonlyArray<T>, index: number, what: string): T => {
    if (index >= values.length) {
        throw new Error(`webgpu: unknown ${what} ${index}`);
    }
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion -- in the list, just checked
    return values[index]!;
};

// The part of a device the calls use, so a test can play them back on a recording.
export type Device = Pick<GPUDevice,
    'createBuffer' | 'createTexture' | 'createSampler' | 'createShaderModule'
    | 'createRenderPipeline' | 'createComputePipeline' | 'createBindGroup' | 'createCommandEncoder'
    | 'queue'>;

// What the passes draw on: the canvas, in its format, and a depth texture as big as it is.
export type Canvas = {
    readonly format: GPUTextureFormat,
    texture: () => GPUTexture,
    depth: (format: GPUTextureFormat) => GPUTexture,
};

// The objects of a context by their ids.
export class Objects {
    public readonly buffers: Map<number, GPUBuffer> = new Map();
    public readonly textures: Map<number, GPUTexture> = new Map();
    public readonly samplers: Map<number, GPUSampler> = new Map();
    public readonly modules: Map<number, GPUShaderModule> = new Map();
    public readonly renderPipelines: Map<number, GPURenderPipeline> = new Map();
    public readonly computePipelines: Map<number, GPUComputePipeline> = new Map();
    public readonly groups: Map<number, GPUBindGroup> = new Map();
}

// Whether `value`, which `getContext` answered, is a WebGPU context.
const isGpuContext = (value: unknown): value is GPUCanvasContext =>
    typeof value === 'object' && value !== null && 'getCurrentTexture' in value;

// The object of `id` in `map`, which wasm made.
const need = <T>(map: Map<number, T>, id: number, what: string): T => {
    const object = map.get(id);
    if (object === undefined) {
        throw new Error(`webgpu: no ${what} ${id}`);
    }
    return object;
};

// Errors of a shader module, told in the console when WebGPU has compiled it.
const report = (module: GPUShaderModule) => {
    module.getCompilationInfo().then((info) => {
        for (const message of info.messages) {
            if (message.type === 'error') {
                console.error(`webgpu: the shader doesn't compile, at ${message.lineNum}:${message.linePos}: ${message.message}`);
            }
        }
    }, () => {});
};

// Plays `bytes` back on `device`, with the objects made so far in `objects`, drawing on `canvas`.
export const play = (device: Device, objects: Objects, canvas: Canvas, bytes: Uint8Array): void => {
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

    const f64 = (): number => {
        const value = data.getFloat64(at, true);
        at += 8;
        return value;
    };

    // A view of the bytes, good while this call runs, as WebGPU copies what it takes.
    const block = (): Uint8Array => {
        const length = u32();
        const value = bytes.subarray(at, at + length);
        at += length;
        return value;
    };

    const text = (): string => decoder.decode(block());

    const format = (): GPUTextureFormat => pick(TEXTURE_FORMATS, u8(), 'texture format') ?? canvas.format;

    const blendComponent = (): GPUBlendComponent => ({
        srcFactor: pick(BLEND_FACTORS, u8(), 'blend factor'),
        dstFactor: pick(BLEND_FACTORS, u8(), 'blend factor'),
        operation: pick(BLEND_OPERATIONS, u8(), 'blend operation'),
    });

    const renderPipeline = (): GPURenderPipelineDescriptor => {
        const vertexModule = need(objects.modules, u32(), 'shader module');
        const vertexEntry = text();
        const buffers: Array<GPUVertexBufferLayout> = [];
        for (let count = u8(); count > 0; count--) {
            const arrayStride = u32();
            const stepMode = pick(STEP_MODES, u8(), 'step mode');
            const attributes: Array<GPUVertexAttribute> = [];
            for (let attribute = u8(); attribute > 0; attribute--) {
                attributes.push({
                    format: pick(VERTEX_FORMATS, u8(), 'vertex format'),
                    offset: u32(),
                    shaderLocation: u32(),
                });
            }
            buffers.push({ arrayStride, stepMode, attributes });
        }

        const topology = pick(TOPOLOGIES, u8(), 'topology');
        const stripIndex = u8();
        const primitive: GPUPrimitiveState = {
            topology,
            stripIndexFormat: stripIndex === 0 ? undefined : pick(INDEX_FORMATS, stripIndex - 1, 'index format'),
            frontFace: pick(FRONT_FACES, u8(), 'front face'),
            cullMode: pick(CULL_MODES, u8(), 'cull mode'),
        };

        const descriptor: GPURenderPipelineDescriptor = {
            layout: 'auto',
            vertex: { module: vertexModule, entryPoint: vertexEntry, buffers },
            primitive,
        };

        if (u8() === 1) {
            descriptor.depthStencil = {
                format: format(),
                depthWriteEnabled: u8() !== 0,
                depthCompare: pick(COMPARE_FUNCTIONS, u8(), 'compare function'),
            };
        }

        if (u8() === 1) {
            const module = need(objects.modules, u32(), 'shader module');
            const entryPoint = text();
            const targets: Array<GPUColorTargetState> = [];
            for (let count = u8(); count > 0; count--) {
                const target: GPUColorTargetState = { format: format() };
                if (u8() === 1) {
                    target.blend = { color: blendComponent(), alpha: blendComponent() };
                }
                target.writeMask = u8();
                targets.push(target);
            }
            descriptor.fragment = { module, entryPoint, targets };
        }

        return descriptor;
    };

    const bindGroup = (): GPUBindGroupDescriptor => {
        const pipelineId = u32();
        const group = u32();
        const pipeline = objects.renderPipelines.get(pipelineId) ?? need(objects.computePipelines, pipelineId, 'pipeline');
        const entries: Array<GPUBindGroupEntry> = [];
        for (let count = u8(); count > 0; count--) {
            const binding = u32();
            const kind = u8();
            if (kind === 0) {
                const buffer = need(objects.buffers, u32(), 'buffer');
                const offset = u32();
                const size = u32();
                entries.push({ binding, resource: { buffer, offset, size: size === 0 ? undefined : size } });
            } else if (kind === 1) {
                entries.push({ binding, resource: need(objects.textures, u32(), 'texture').createView() });
            } else {
                entries.push({ binding, resource: need(objects.samplers, u32(), 'sampler') });
            }
        }
        return { layout: pipeline.getBindGroupLayout(group), entries };
    };

    const renderPass = (): GPURenderPassDescriptor => {
        const colorAttachments: Array<GPURenderPassColorAttachment> = [];
        for (let count = u8(); count > 0; count--) {
            const texture = u8() === 0 ? canvas.texture() : need(objects.textures, u32(), 'texture');
            const clear = u8() === 1;
            const clearValue = clear ? { r: f64(), g: f64(), b: f64(), a: f64() } : undefined;
            colorAttachments.push({
                view: texture.createView(),
                loadOp: clear ? 'clear' : 'load',
                clearValue,
                storeOp: u8() === 0 ? 'store' : 'discard',
            });
        }

        const descriptor: GPURenderPassDescriptor = { colorAttachments };
        if (u8() === 1) {
            const texture = u8() === 0 ? canvas.depth(format()) : need(objects.textures, u32(), 'texture');
            const clear = u8() === 1;
            const depthClearValue = clear ? f32() : undefined;
            const depthStoreOp = u8() === 0 ? 'store' : 'discard';
            descriptor.depthStencilAttachment = {
                view: texture.createView(),
                depthLoadOp: clear ? 'clear' : 'load',
                depthClearValue,
                depthStoreOp,
                // A format with stencil wants what becomes of it.
                ...(texture.format === 'depth24plus-stencil8'
                    ? { stencilLoadOp: 'clear', stencilClearValue: 0, stencilStoreOp: depthStoreOp }
                    : {}),
            };
        }
        return descriptor;
    };

    let encoder: GPUCommandEncoder | null = null;
    let render: GPURenderPassEncoder | null = null;
    let compute: GPUComputePassEncoder | null = null;
    // Let go during the batch, destroyed after it.
    const doomed: Array<GPUBuffer | GPUTexture> = [];

    const encoding = (): GPUCommandEncoder => {
        encoder ??= device.createCommandEncoder();
        return encoder;
    };

    const end = () => {
        render?.end();
        compute?.end();
        render = null;
        compute = null;
    };

    // The passes so far to the queue.
    const submit = () => {
        end();
        if (encoder !== null) {
            device.queue.submit([encoder.finish()]);
            encoder = null;
        }
    };

    // Writes come after the passes before them, unless they come during a pass.
    const write = () => {
        if (render === null && compute === null) {
            submit();
        }
    };

    const rendering = (): GPURenderPassEncoder => {
        if (render === null) {
            throw new Error('webgpu: no render pass');
        }
        return render;
    };

    const passing = (): GPURenderPassEncoder | GPUComputePassEncoder => {
        const pass = render ?? compute;
        if (pass === null) {
            throw new Error('webgpu: no pass');
        }
        return pass;
    };

    while (at < bytes.length) {
        const op = u8();

        switch (op) {
            case Op.Context:
                // Taken by `WebGpus.run` when it made the context: its options and its callback.
                at += 10;
                break;
            case Op.CreateBuffer: {
                const id = u32();
                const size = u32();
                objects.buffers.set(id, device.createBuffer({ size, usage: u32() }));
                break;
            }
            case Op.DestroyBuffer: {
                const id = u32();
                const buffer = objects.buffers.get(id);
                if (buffer !== undefined) {
                    doomed.push(buffer);
                }
                objects.buffers.delete(id);
                break;
            }
            case Op.WriteBuffer: {
                const buffer = need(objects.buffers, u32(), 'buffer');
                const offset = u32();
                write();
                device.queue.writeBuffer(buffer, offset, block());
                break;
            }
            case Op.CreateTexture: {
                const id = u32();
                const width = u32();
                const height = u32();
                const textureFormat = format();
                objects.textures.set(id, device.createTexture({ size: [width, height], format: textureFormat, usage: u32() }));
                break;
            }
            case Op.DestroyTexture: {
                const id = u32();
                const texture = objects.textures.get(id);
                if (texture !== undefined) {
                    doomed.push(texture);
                }
                objects.textures.delete(id);
                break;
            }
            case Op.WriteTexture: {
                const texture = need(objects.textures, u32(), 'texture');
                const bytesPerRow = u32();
                const width = u32();
                const height = u32();
                write();
                device.queue.writeTexture({ texture }, block(), { bytesPerRow }, [width, height]);
                break;
            }
            case Op.CreateSampler: {
                const id = u32();
                objects.samplers.set(id, device.createSampler({
                    addressModeU: pick(ADDRESS_MODES, u8(), 'address mode'),
                    addressModeV: pick(ADDRESS_MODES, u8(), 'address mode'),
                    magFilter: pick(FILTER_MODES, u8(), 'filter mode'),
                    minFilter: pick(FILTER_MODES, u8(), 'filter mode'),
                    mipmapFilter: pick(FILTER_MODES, u8(), 'filter mode'),
                }));
                break;
            }
            case Op.DeleteSampler:
                objects.samplers.delete(u32());
                break;
            case Op.CreateShaderModule: {
                const id = u32();
                const module = device.createShaderModule({ code: text() });
                report(module);
                objects.modules.set(id, module);
                break;
            }
            case Op.DeleteShaderModule:
                objects.modules.delete(u32());
                break;
            case Op.CreateRenderPipeline: {
                const id = u32();
                objects.renderPipelines.set(id, device.createRenderPipeline(renderPipeline()));
                break;
            }
            case Op.CreateComputePipeline: {
                const id = u32();
                const module = need(objects.modules, u32(), 'shader module');
                objects.computePipelines.set(id, device.createComputePipeline({
                    layout: 'auto',
                    compute: { module, entryPoint: text() },
                }));
                break;
            }
            case Op.DeletePipeline: {
                const id = u32();
                objects.renderPipelines.delete(id);
                objects.computePipelines.delete(id);
                break;
            }
            case Op.CreateBindGroup: {
                const id = u32();
                objects.groups.set(id, device.createBindGroup(bindGroup()));
                break;
            }
            case Op.DeleteBindGroup:
                objects.groups.delete(u32());
                break;
            case Op.BeginRenderPass:
                end();
                render = encoding().beginRenderPass(renderPass());
                break;
            case Op.BeginComputePass:
                end();
                compute = encoding().beginComputePass();
                break;
            case Op.EndPass:
                end();
                break;
            case Op.SetPipeline: {
                const id = u32();
                if (render !== null) {
                    render.setPipeline(need(objects.renderPipelines, id, 'render pipeline'));
                } else if (compute !== null) {
                    compute.setPipeline(need(objects.computePipelines, id, 'compute pipeline'));
                } else {
                    throw new Error('webgpu: no pass');
                }
                break;
            }
            case Op.SetBindGroup: {
                const index = u32();
                passing().setBindGroup(index, need(objects.groups, u32(), 'bind group'));
                break;
            }
            case Op.SetVertexBuffer: {
                const slot = u32();
                const buffer = need(objects.buffers, u32(), 'buffer');
                rendering().setVertexBuffer(slot, buffer, u32());
                break;
            }
            case Op.SetIndexBuffer: {
                const buffer = need(objects.buffers, u32(), 'buffer');
                const indexFormat = pick(INDEX_FORMATS, u8(), 'index format');
                rendering().setIndexBuffer(buffer, indexFormat, u32());
                break;
            }
            case Op.Draw:
                rendering().draw(u32(), u32(), u32(), u32());
                break;
            case Op.DrawIndexed:
                rendering().drawIndexed(u32(), u32(), u32(), i32(), u32());
                break;
            case Op.DispatchWorkgroups: {
                if (compute === null) {
                    throw new Error('webgpu: no compute pass');
                }
                compute.dispatchWorkgroups(u32(), u32(), u32());
                break;
            }
            case Op.SetViewport:
                rendering().setViewport(f32(), f32(), f32(), f32(), f32(), f32());
                break;
            case Op.SetScissorRect:
                rendering().setScissorRect(u32(), u32(), u32(), u32());
                break;
            default:
                throw new Error(`webgpu: unknown operation ${op}`);
        }
    }

    submit();
    for (const object of doomed) {
        object.destroy();
    }
};

// A canvas's context: the calls waiting for its device, which may be lost, its objects, and its
// depth texture.
class Context {
    public objects: Objects = new Objects();
    public state: 'starting' | 'ready' | 'lost' | 'unavailable' = 'starting';
    public waiting: Array<Uint8Array> = [];
    public device: GPUDevice | null = null;
    public gpu: GPUCanvasContext | null = null;
    public format: GPUTextureFormat = 'bgra8unorm';
    public depth: GPUTexture | null = null;

    constructor(
        public readonly callback: number,
        public readonly alphaMode: GPUCanvasAlphaMode,
        public readonly powerPreference: GPUPowerPreference | undefined,
    ) {}
}

export class WebGpus {
    private readonly contexts: WeakMap<HTMLCanvasElement, Context> = new WeakMap();
    private readonly sizes: CanvasSizes = new CanvasSizes((canvas: HTMLCanvasElement, size: Size) => {
        const context = this.contexts.get(canvas);
        if (context === undefined) {
            return false;
        }
        this.told(context, [0, size.width, size.height]);
        return true;
    });

    // `gpu` is the browser's WebGPU, if it has it.
    public constructor(
        private readonly tell: Tell,
        private readonly gpu: () => GPU | undefined = () => ('gpu' in navigator ? navigator.gpu : undefined),
    ) {}

    // `bytes` is good only for the call it came in (see `DriverDom.update`): it is played now, or
    // copied to wait for the device.
    public run(node: Element, bytes: Uint8Array) {
        if (!(node instanceof HTMLCanvasElement)) {
            throw new Error('webgpu: the element is not a canvas');
        }

        let context = this.contexts.get(node);
        if (context === undefined) {
            context = this.make(bytes);
            this.contexts.set(node, context);
            this.sizes.observe(node);
            this.start(node, context, false);
        }

        if (context.state === 'starting') {
            context.waiting.push(bytes.slice());
        } else if (context.state === 'ready') {
            this.play(node, context, bytes);
        }
    }

    public forget(node: Element | Comment | Text) {
        if (node instanceof HTMLCanvasElement) {
            const context = this.contexts.get(node);
            if (context !== undefined) {
                this.contexts.delete(node);
                this.sizes.forget(node);
                context.device?.destroy();
            }
        }
    }

    // The context, from the first calls of it: they begin with its options and its callback.
    private make(bytes: Uint8Array): Context {
        const first = bytes[0] === Op.Context && bytes.length >= 11;
        const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const alphaMode = ALPHA_MODES[first ? data.getUint8(1) : 0] ?? 'opaque';
        const powerPreference = POWER_PREFERENCES[first ? data.getUint8(2) : 0];
        const callback = first ? data.getUint32(3, true) + data.getUint32(7, true) * 2 ** 32 : 0;
        return new Context(callback, alphaMode, powerPreference);
    }

    // Asks for a device, for the first time or after the last was lost.
    private start(canvas: HTMLCanvasElement, context: Context, again: boolean) {
        const gpu = this.gpu();
        if (gpu === undefined) {
            this.started(canvas, context, null, again);
            return;
        }

        context.format = gpu.getPreferredCanvasFormat();
        const options = context.powerPreference === undefined ? {} : { powerPreference: context.powerPreference };
        gpu.requestAdapter(options)
            .then((adapter) => adapter?.requestDevice() ?? null)
            .then(
                (device) => { this.started(canvas, context, device, again); },
                (error: unknown) => {
                    console.error(error);
                    this.started(canvas, context, null, again);
                },
            );
    }

    private started(canvas: HTMLCanvasElement, context: Context, device: GPUDevice | null, again: boolean) {
        if (this.contexts.get(canvas) !== context) {
            device?.destroy();
            return;
        }

        if (context.gpu === null && device !== null) {
            const gpu = canvas.getContext('webgpu');
            context.gpu = isGpuContext(gpu) ? gpu : null;
        }
        if (device === null || context.gpu === null) {
            device?.destroy();
            context.state = 'unavailable';
            context.waiting = [];
            console.error('webgpu: the browser has no WebGPU, or no GPU for it, or the canvas has another context');
            this.told(context, [2]);
            return;
        }

        context.device = device;
        context.gpu.configure({ device, format: context.format, alphaMode: context.alphaMode });
        device.onuncapturederror = (event) => {
            console.error(`webgpu: ${event.error.message}`);
        };
        void device.lost.then((info) => {
            this.lost(canvas, context, device, info);
        });

        context.state = 'ready';
        const waiting = context.waiting;
        context.waiting = [];
        for (const bytes of waiting) {
            try {
                this.play(canvas, context, bytes);
            } catch (error) {
                console.error(error);
            }
        }

        if (again) {
            this.told(context, [1]);
        }
    }

    // Unless it was let go with its canvas, another is asked for.
    private lost(canvas: HTMLCanvasElement, context: Context, device: GPUDevice, info: GPUDeviceLostInfo) {
        if (this.contexts.get(canvas) !== context || context.device !== device) {
            return;
        }

        // Not an error of the page's, which gets another device.
        console.warn(`webgpu: the device was lost, ${info.message}`);
        context.state = 'lost';
        context.device = null;
        context.depth = null;
        context.objects = new Objects();
        this.start(canvas, context, true);
    }

    private play(canvas: HTMLCanvasElement, context: Context, bytes: Uint8Array) {
        const { device, gpu } = context;
        if (device === null || gpu === null) {
            return;
        }

        this.sizes.fit(canvas);
        play(device, context.objects, {
            format: context.format,
            texture: () => gpu.getCurrentTexture(),
            depth: (format) => {
                const depth = context.depth;
                if (depth !== null && depth.format === format
                    && depth.width === canvas.width && depth.height === canvas.height) {
                    return depth;
                }
                depth?.destroy();
                context.depth = device.createTexture({
                    size: [canvas.width, canvas.height],
                    format,
                    usage: RENDER_ATTACHMENT,
                });
                return context.depth;
            },
        }, bytes);
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
}
