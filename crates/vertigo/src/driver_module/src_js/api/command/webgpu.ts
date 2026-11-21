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
// Reads copy what they read in the command buffer, and map the copy once it went to the queue.
//
// The bitmap of a canvas is kept as big as the canvas is on the screen, times the scale it was
// asked for, see `canvas_sizes.ts`, and so are a depth texture of it and a multisampled one, made
// again when the bitmap is resized. A new scale is told as a new size.
//
// A texture of an image from a URL is a white pixel until the image comes, when it becomes another
// texture, as big as the image: the bind groups with it are made again then, from what they were
// made of, which is kept for them.
//
// A device lost is asked for again. When one comes, every object is gone: the ids are forgotten,
// and wasm is told, to make them again. Wasm is told `[0, width, height]` when the canvas changes
// its size, `[1]` when the context has a new device, `[2]` when there is no WebGPU for it,
// `[3, texture, width, height]` when an image is in its texture, `[4, texture]` when it didn't
// load, `[5, request, bytes]` with the bytes a read read, `[6, request, width, height, bytes]` with
// the pixels, see `GpuEvent` in webgpu.rs.

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
    CreateBindGroupLayout: 29,
    DeleteBindGroupLayout: 30,
    CreatePipelineLayout: 31,
    DeletePipelineLayout: 32,
    PipelineBindGroupLayout: 33,
    GenerateMipmaps: 34,
    TextureFromUrl: 35,
    ReadBuffer: 36,
    ReadPixels: 37,
    BitmapScale: 38,
} as const;

// Bytes of `Op.Context` after its code: the alpha mode, the power preference, the callback (8)
// and the scale of the bitmap (4).
const CONTEXT_LENGTH = 14;

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
// The bindings of layouts, by their kinds, see `BindingType::kind` in webgpu.rs.
const BINDINGS: ReadonlyArray<(multisampled: boolean) => Omit<GPUBindGroupLayoutEntry, 'binding' | 'visibility'>> = [
    () => ({ buffer: { type: 'uniform' } }),
    () => ({ buffer: { type: 'storage' } }),
    () => ({ buffer: { type: 'read-only-storage' } }),
    () => ({ sampler: { type: 'filtering' } }),
    () => ({ sampler: { type: 'non-filtering' } }),
    () => ({ sampler: { type: 'comparison' } }),
    (multisampled) => ({ texture: { sampleType: 'float', multisampled } }),
    (multisampled) => ({ texture: { sampleType: 'unfilterable-float', multisampled } }),
    (multisampled) => ({ texture: { sampleType: 'depth', multisampled } }),
    (multisampled) => ({ texture: { sampleType: 'sint', multisampled } }),
    (multisampled) => ({ texture: { sampleType: 'uint', multisampled } }),
];

// Bytes of a pixel of the formats of other than 4, see `TextureFormat::bytes_per_pixel`.
const BYTES_PER_PIXEL: Partial<Record<GPUTextureFormat, number>> = {
    'r8unorm': 1, 'rg8unorm': 2, 'r16float': 2, 'depth16unorm': 2, 'rgba16float': 8,
    'rg32float': 8, 'rgba32float': 16,
};

// `GPUTextureUsage` and `GPUBufferUsage`, which aren't there outside of a browser.
const COPY_SRC = 0x01;
const COPY_DST = 0x02;
const TEXTURE_BINDING = 0x04;
const RENDER_ATTACHMENT = 0x10;
const MAP_READ = 0x01;
const BUFFER_COPY_DST = 0x08;
// `GPUMapMode.READ`
const READ = 0x01;

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
    | 'createRenderPipeline' | 'createComputePipeline' | 'createBindGroup' | 'createBindGroupLayout'
    | 'createPipelineLayout' | 'createCommandEncoder' | 'queue'>;

// How an image goes into a texture, see `ImageOptions` in webgpu.rs.
export type ImageOptions = { mipmaps: boolean, premultiplied: boolean, flip: boolean };

// What the passes draw on, and what is done for the calls beyond WebGPU's own: the canvas, in
// its format, a depth texture and a multisampled one as big as it, the smaller images of a
// texture, an image loaded into a texture, and wasm told of what was read.
export type Canvas = {
    readonly format: GPUTextureFormat,
    texture: () => GPUTexture,
    depth: (format: GPUTextureFormat, samples: number) => GPUTexture,
    multisampled: (samples: number) => GPUTexture,
    mipmaps: (texture: GPUTexture) => void,
    load: (texture: number, url: string, options: ImageOptions) => void,
    tell: (event: Array<number | Uint8Array<ArrayBuffer>>) => void,
    // A new scale of the bitmap.
    rescale: (scale: number) => void,
};

// What a bind group is made of, to be made again when a texture of it becomes another: its
// textures by their ids, and with what they were then, in case they were let go since.
type GroupSource = {
    layout: GPUBindGroupLayout,
    entries: Array<{ binding: number, resource: GPUBindingResource } | { binding: number, texture: number, was: GPUTexture }>,
};

// The objects of a context by their ids.
export class Objects {
    public readonly buffers: Map<number, GPUBuffer> = new Map();
    public readonly textures: Map<number, GPUTexture> = new Map();
    public readonly samplers: Map<number, GPUSampler> = new Map();
    public readonly modules: Map<number, GPUShaderModule> = new Map();
    public readonly renderPipelines: Map<number, GPURenderPipeline> = new Map();
    public readonly computePipelines: Map<number, GPUComputePipeline> = new Map();
    public readonly layouts: Map<number, GPUBindGroupLayout> = new Map();
    public readonly pipelineLayouts: Map<number, GPUPipelineLayout> = new Map();
    public readonly groups: Map<number, GPUBindGroup> = new Map();
    public readonly groupSources: Map<number, GroupSource> = new Map();

    // The bind group of `id`, made of `source`, now or again.
    public makeGroup(device: Device, id: number, source: GroupSource) {
        const entries: Array<GPUBindGroupEntry> = source.entries.map((entry) => 'texture' in entry
            ? { binding: entry.binding, resource: (this.textures.get(entry.texture) ?? entry.was).createView() }
            : entry);
        this.groups.set(id, device.createBindGroup({ layout: source.layout, entries }));
        this.groupSources.set(id, source);
    }

    // The texture of `id` is `texture` now: the bind groups with it are made again.
    public replaceTexture(device: Device, id: number, texture: GPUTexture) {
        this.textures.set(id, texture);
        for (const [group, source] of this.groupSources) {
            if (source.entries.some((entry) => 'texture' in entry && entry.texture === id)) {
                this.makeGroup(device, group, source);
            }
        }
    }
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

    // The layout of a pipeline: 0 for the one WebGPU finds in its shaders.
    const pipelineLayout = (): GPUPipelineLayout | 'auto' => {
        const id = u32();
        return id === 0 ? 'auto' : need(objects.pipelineLayouts, id, 'pipeline layout');
    };

    const renderPipeline = (): GPURenderPipelineDescriptor => {
        const layout = pipelineLayout();
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
            layout,
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

        const samples = u8();
        if (samples > 1) {
            descriptor.multisample = { count: samples };
        }

        return descriptor;
    };

    const groupSource = (): GroupSource => {
        const layout = need(objects.layouts, u32(), 'bind group layout');
        const entries: GroupSource['entries'] = [];
        for (let count = u8(); count > 0; count--) {
            const binding = u32();
            const kind = u8();
            const id = u32();
            if (kind === 0) {
                const buffer = need(objects.buffers, id, 'buffer');
                const offset = u32();
                const size = u32();
                entries.push({ binding, resource: { buffer, offset, size: size === 0 ? undefined : size } });
            } else if (kind === 1) {
                entries.push({ binding, texture: id, was: need(objects.textures, id, 'texture') });
            } else {
                entries.push({ binding, resource: need(objects.samplers, id, 'sampler') });
            }
        }
        return { layout, entries };
    };

    // Where colors go: the canvas, a texture, or a multisampled one as big as the canvas.
    const target = (): GPUTexture => {
        const kind = u8();
        if (kind === 0) {
            return canvas.texture();
        } else if (kind === 1) {
            return need(objects.textures, u32(), 'texture');
        }
        return canvas.multisampled(u8());
    };

    const renderPass = (): GPURenderPassDescriptor => {
        const colorAttachments: Array<GPURenderPassColorAttachment> = [];
        // A depth texture of the canvas has as many samples as the first color attachment.
        let samples = 1;
        for (let count = u8(); count > 0; count--) {
            const texture = target();
            if (colorAttachments.length === 0) {
                samples = texture.sampleCount;
            }
            const resolveTarget = u8() === 1 ? target().createView() : undefined;
            const clear = u8() === 1;
            const clearValue = clear ? { r: f64(), g: f64(), b: f64(), a: f64() } : undefined;
            colorAttachments.push({
                view: texture.createView(),
                resolveTarget,
                loadOp: clear ? 'clear' : 'load',
                clearValue,
                storeOp: u8() === 0 ? 'store' : 'discard',
            });
        }

        const descriptor: GPURenderPassDescriptor = { colorAttachments };
        if (u8() === 1) {
            const texture = u8() === 0 ? canvas.depth(format(), samples) : need(objects.textures, u32(), 'texture');
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
    // Reads, to be mapped once their copies went to the queue.
    const reads: Array<() => void> = [];

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
        for (const read of reads.splice(0)) {
            read();
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

    // A buffer `size` bytes long the GPU copies into, mapped, to `done` with its bytes.
    const readInto = (size: number, copy: (into: GPUBuffer) => void, done: (bytes: Uint8Array<ArrayBuffer>) => void) => {
        const into = device.createBuffer({ size, usage: MAP_READ | BUFFER_COPY_DST });
        copy(into);
        reads.push(() => {
            into.mapAsync(READ).then(() => {
                done(new Uint8Array(into.getMappedRange().slice(0)));
                into.destroy();
            }, () => {
                into.destroy();
            });
        });
    };

    while (at < bytes.length) {
        const op = u8();

        switch (op) {
            case Op.Context:
                // Taken by `WebGpus.run` when it made the context: its options and its callback.
                at += CONTEXT_LENGTH;
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
                const usage = u32();
                const mipLevelCount = u32();
                const sampleCount = u8();
                objects.textures.set(id, device.createTexture({
                    size: [width, height], format: textureFormat, usage, mipLevelCount, sampleCount,
                }));
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
                const layout = pipelineLayout();
                const module = need(objects.modules, u32(), 'shader module');
                objects.computePipelines.set(id, device.createComputePipeline({
                    layout,
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
                objects.makeGroup(device, id, groupSource());
                break;
            }
            case Op.DeleteBindGroup: {
                const id = u32();
                objects.groups.delete(id);
                objects.groupSources.delete(id);
                break;
            }
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
            case Op.CreateBindGroupLayout: {
                const id = u32();
                const entries: Array<GPUBindGroupLayoutEntry> = [];
                for (let count = u8(); count > 0; count--) {
                    const binding = u32();
                    const visibility = u8();
                    const kind = pick(BINDINGS, u8(), 'binding');
                    entries.push({ binding, visibility, ...kind(u8() !== 0) });
                }
                objects.layouts.set(id, device.createBindGroupLayout({ entries }));
                break;
            }
            case Op.DeleteBindGroupLayout:
                objects.layouts.delete(u32());
                break;
            case Op.CreatePipelineLayout: {
                const id = u32();
                const bindGroupLayouts: Array<GPUBindGroupLayout> = [];
                for (let count = u8(); count > 0; count--) {
                    bindGroupLayouts.push(need(objects.layouts, u32(), 'bind group layout'));
                }
                objects.pipelineLayouts.set(id, device.createPipelineLayout({ bindGroupLayouts }));
                break;
            }
            case Op.DeletePipelineLayout:
                objects.pipelineLayouts.delete(u32());
                break;
            case Op.PipelineBindGroupLayout: {
                const id = u32();
                const pipelineId = u32();
                const pipeline = objects.renderPipelines.get(pipelineId) ?? need(objects.computePipelines, pipelineId, 'pipeline');
                objects.layouts.set(id, pipeline.getBindGroupLayout(u32()));
                break;
            }
            case Op.GenerateMipmaps: {
                const texture = need(objects.textures, u32(), 'texture');
                // After what was written into its first level
                write();
                canvas.mipmaps(texture);
                break;
            }
            case Op.TextureFromUrl: {
                const id = u32();
                const options = { mipmaps: u8() !== 0, premultiplied: u8() !== 0, flip: u8() !== 0 };
                const url = text();
                const white = device.createTexture({
                    size: [1, 1], format: 'rgba8unorm', usage: TEXTURE_BINDING | COPY_DST | COPY_SRC,
                });
                device.queue.writeTexture({ texture: white }, new Uint8Array([255, 255, 255, 255]), { bytesPerRow: 4 }, [1, 1]);
                objects.textures.set(id, white);
                canvas.load(id, url, options);
                break;
            }
            case Op.ReadBuffer: {
                const request = u32();
                const buffer = need(objects.buffers, u32(), 'buffer');
                end();
                readInto(buffer.size, (into) => {
                    encoding().copyBufferToBuffer(buffer, 0, into, 0, buffer.size);
                }, (read) => {
                    canvas.tell([5, request, read]);
                });
                break;
            }
            case Op.BitmapScale:
                canvas.rescale(f32());
                break;
            case Op.ReadPixels: {
                const request = u32();
                const kind = u8();
                if (kind === 2) {
                    throw new Error('webgpu: a multisampled target is read once resolved');
                }
                const texture = kind === 0 ? canvas.texture() : need(objects.textures, u32(), 'texture');
                const swap = kind === 0 && texture.format.startsWith('bgra');
                const [width, height] = [texture.width, texture.height];
                // A row of the copy is a multiple of 256 bytes.
                const row = width * (BYTES_PER_PIXEL[texture.format] ?? 4);
                const bytesPerRow = Math.ceil(row / 256) * 256;
                end();
                readInto(bytesPerRow * height, (into) => {
                    encoding().copyTextureToBuffer({ texture }, { buffer: into, bytesPerRow }, [width, height]);
                }, (read) => {
                    const pixels = new Uint8Array(row * height);
                    for (let y = 0; y < height; y++) {
                        pixels.set(read.subarray(y * bytesPerRow, y * bytesPerRow + row), y * row);
                    }
                    if (swap) {
                        for (let pixel = 0; pixel < pixels.length; pixel += 4) {
                            const blue = pixels[pixel] ?? 0;
                            pixels[pixel] = pixels[pixel + 2] ?? 0;
                            pixels[pixel + 2] = blue;
                        }
                    }
                    canvas.tell([6, request, width, height, pixels]);
                });
                break;
            }
            default:
                throw new Error(`webgpu: unknown operation ${op}`);
        }
    }

    submit();
    for (const object of doomed) {
        object.destroy();
    }
};

// A pipeline making the next smaller image of a texture of a format, and its sampler, see
// `generateMipmaps`.
type Mipmapper = { pipeline: GPURenderPipeline, sampler: GPUSampler };

// A triangle over the whole target, reading the level before between its pixels.
const MIPMAP_SHADER = `
struct Corner {
    @builtin(position) position: vec4f,
    @location(0) at: vec2f,
};

@group(0) @binding(0) var level: texture_2d<f32>;
@group(0) @binding(1) var between: sampler;

@vertex fn vertex(@builtin(vertex_index) index: u32) -> Corner {
    let corner = vec2f(f32((index << 1u) & 2u), f32(index & 2u));
    return Corner(vec4f(corner * 2.0 - 1.0, 0.0, 1.0), vec2f(corner.x, 1.0 - corner.y));
}

@fragment fn fragment(corner: Corner) -> @location(0) vec4f {
    return textureSample(level, between, corner.at);
}
`;

// Draws each level of `texture` after the first from the one before it, half as big, with the
// pipeline for its format, made once a device, kept in `made`.
const generateMipmaps = (device: GPUDevice, made: Map<GPUTextureFormat, Mipmapper>, texture: GPUTexture) => {
    let mipmapper = made.get(texture.format);
    if (mipmapper === undefined) {
        const module = device.createShaderModule({ code: MIPMAP_SHADER });
        mipmapper = {
            pipeline: device.createRenderPipeline({
                layout: 'auto',
                vertex: { module, entryPoint: 'vertex' },
                fragment: { module, entryPoint: 'fragment', targets: [{ format: texture.format }] },
            }),
            sampler: device.createSampler({ minFilter: 'linear', magFilter: 'linear' }),
        };
        made.set(texture.format, mipmapper);
    }

    const encoder = device.createCommandEncoder();
    for (let level = 1; level < texture.mipLevelCount; level++) {
        const group = device.createBindGroup({
            layout: mipmapper.pipeline.getBindGroupLayout(0),
            entries: [
                { binding: 0, resource: texture.createView({ baseMipLevel: level - 1, mipLevelCount: 1 }) },
                { binding: 1, resource: mipmapper.sampler },
            ],
        });
        const pass = encoder.beginRenderPass({
            colorAttachments: [{
                view: texture.createView({ baseMipLevel: level, mipLevelCount: 1 }),
                loadOp: 'clear',
                storeOp: 'store',
            }],
        });
        pass.setPipeline(mipmapper.pipeline);
        pass.setBindGroup(0, group);
        pass.draw(3);
        pass.end();
    }
    device.queue.submit([encoder.finish()]);
};

// How many levels a texture of `width` by `height` has, down to a pixel.
const levels = (width: number, height: number): number => Math.floor(Math.log2(Math.max(width, height, 1))) + 1;

// A canvas's context: the calls waiting for its device, which may be lost, its objects, and the
// textures as big as the canvas.
class Context {
    public objects: Objects = new Objects();
    // `forgotten` with its canvas: what comes for it after, f. ex. an image or a read, is told
    // nobody, as wasm has let go of its callback.
    public state: 'starting' | 'ready' | 'lost' | 'unavailable' | 'forgotten' = 'starting';
    public waiting: Array<Uint8Array> = [];
    public device: GPUDevice | null = null;
    public gpu: GPUCanvasContext | null = null;
    public format: GPUTextureFormat = 'bgra8unorm';
    public depth: GPUTexture | null = null;
    public multisampled: GPUTexture | null = null;
    public mipmappers: Map<GPUTextureFormat, Mipmapper> = new Map();
    // The URL of the image each texture waits for, by its id: only the last asked for goes in.
    public readonly loading: Map<number, string> = new Map();

    constructor(
        public readonly callback: number,
        public readonly alphaMode: GPUCanvasAlphaMode,
        public readonly powerPreference: GPUPowerPreference | undefined,
        // Pixels of the bitmap to a device pixel, see `WebGpuOptions::bitmap_scale`.
        public readonly scale: number,
    ) {}
}

// The scale asked for, 1 for one not above 0 nor finite.
const scaleOf = (asked: number): number => asked > 0 && Number.isFinite(asked) ? asked : 1;

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
            this.sizes.observe(node, context.scale);
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
                context.state = 'forgotten';
                context.device?.destroy();
            }
        }
    }

    // The context, from the first calls of it: they begin with its options, its callback and the
    // scale of its bitmap.
    private make(bytes: Uint8Array): Context {
        const first = bytes[0] === Op.Context && bytes.length > CONTEXT_LENGTH;
        const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const alphaMode = ALPHA_MODES[first ? data.getUint8(1) : 0] ?? 'opaque';
        const powerPreference = POWER_PREFERENCES[first ? data.getUint8(2) : 0];
        const callback = first ? data.getUint32(3, true) + data.getUint32(7, true) * 2 ** 32 : 0;
        const scale = scaleOf(first ? data.getFloat32(11, true) : 1);
        return new Context(callback, alphaMode, powerPreference, scale);
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
            // Not an error of the page's, which is told, to draw another way.
            console.warn('webgpu: the browser has no WebGPU, or no GPU for it, or the canvas has another context');
            this.told(context, [2]);
            return;
        }

        context.device = device;
        // Read back too, see `Op.ReadPixels`
        context.gpu.configure({
            device, format: context.format, alphaMode: context.alphaMode, usage: RENDER_ATTACHMENT | COPY_SRC,
        });
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
        context.multisampled = null;
        context.mipmappers = new Map();
        context.loading.clear();
        context.objects = new Objects();
        this.start(canvas, context, true);
    }

    private play(canvas: HTMLCanvasElement, context: Context, bytes: Uint8Array) {
        const { device, gpu } = context;
        if (device === null || gpu === null) {
            return;
        }

        this.sizes.fit(canvas);
        // A texture as big as the canvas, of `format` and `samples`, made again when they change.
        const sized = (kept: GPUTexture | null, format: GPUTextureFormat, samples: number): GPUTexture => {
            if (kept !== null && kept.format === format && kept.sampleCount === samples
                && kept.width === canvas.width && kept.height === canvas.height) {
                return kept;
            }
            kept?.destroy();
            return device.createTexture({
                size: [canvas.width, canvas.height],
                format,
                sampleCount: samples,
                usage: RENDER_ATTACHMENT,
            });
        };
        play(device, context.objects, {
            format: context.format,
            texture: () => gpu.getCurrentTexture(),
            depth: (format, samples) => {
                context.depth = sized(context.depth, format, samples);
                return context.depth;
            },
            multisampled: (samples) => {
                context.multisampled = sized(context.multisampled, context.format, samples);
                return context.multisampled;
            },
            mipmaps: (texture) => {
                generateMipmaps(device, context.mipmappers, texture);
            },
            load: (texture, url, options) => {
                this.load(context, texture, url, options);
            },
            tell: (event) => {
                this.told(context, event);
            },
            rescale: (scale) => {
                this.sizes.rescale(canvas, scaleOf(scale));
            },
        }, bytes);
    }

    // Puts the image at `url` into a new texture for `texture` when it comes, unless the texture
    // was let go meanwhile, or asked for another image, or the device was lost.
    private load(context: Context, texture: number, url: string, options: ImageOptions) {
        const objects = context.objects;
        context.loading.set(texture, url);
        const current = () => context.objects === objects && context.loading.get(texture) === url
            && objects.textures.has(texture);

        const image = new Image();
        image.crossOrigin = 'anonymous';
        image.onload = () => {
            const device = context.device;
            if (!current() || device === null) {
                return;
            }
            context.loading.delete(texture);
            const [width, height] = [image.naturalWidth, image.naturalHeight];
            const made = device.createTexture({
                size: [width, height],
                format: 'rgba8unorm',
                usage: TEXTURE_BINDING | COPY_DST | COPY_SRC | RENDER_ATTACHMENT,
                mipLevelCount: options.mipmaps ? levels(width, height) : 1,
            });
            device.queue.copyExternalImageToTexture(
                { source: image, flipY: options.flip },
                { texture: made, premultipliedAlpha: options.premultiplied },
                [width, height],
            );
            if (options.mipmaps) {
                generateMipmaps(device, context.mipmappers, made);
            }
            objects.textures.get(texture)?.destroy();
            objects.replaceTexture(device, texture, made);
            this.told(context, [3, texture, width, height]);
        };
        image.onerror = () => {
            if (!current()) {
                return;
            }
            context.loading.delete(texture);
            console.error(`webgpu: the image ${url} didn't load`);
            this.told(context, [4, texture]);
        };
        image.src = url;
    }

    private told(context: Context, event: Array<number | Uint8Array<ArrayBuffer>>) {
        if (context.callback === 0 || context.state === 'forgotten') {
            return;
        }
        try {
            this.tell(context.callback, event);
        } catch (error) {
            console.error(error);
        }
    }
}
