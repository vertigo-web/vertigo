// WebGPU calls played back on a device, against the bytes `crates/vertigo/src/webgpu.rs` makes.

import { Canvas, Device, Objects, WebGpus, play } from "./webgpu";

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

// A value as the log shows it: an object WebGPU made by its name, bytes as a list, a descriptor
// as JSON of those.
const shown = (value: unknown): string => {
    if (typeof value === 'object' && value !== null && 'name' in value) {
        return String(value.name);
    }
    return JSON.stringify(value, (key, inner: unknown) => {
        if (inner instanceof Uint8Array) {
            return Array.from(inner);
        }
        if (key !== '' && typeof inner === 'object' && inner !== null && 'name' in inner) {
            return inner.name;
        }
        return inner;
    });
};

// Writes down each call of `target`, named `prefix` in the log, answered by `answers`.
const recorder = (log: Array<string>, prefix: string, answers: Record<string, (...args: Array<any>) => unknown>, extra: Record<string, unknown> = {}) =>
    new Proxy(extra, {
        get: (target, name: string) => {
            // Not a promise, nor JSON of its own.
            if (name in target || name === 'then' || name === 'toJSON') {
                return target[name];
            }
            return (...args: Array<unknown>) => {
                log.push(`${prefix}${name}(${args.map(shown).join(',')})`);
                return answers[name]?.(...args);
            };
        },
    });

// A device writing down each call into `log`. The objects it makes are named by what they are and
// a number.
const recording = (log: Array<string>): Device & { destroy: () => void } => {
    let made = 0;
    const named = (kind: string, extra: Record<string, unknown> = {}) => ({ name: `${kind}${++made}`, ...extra });
    const destroyable = (kind: string, extra: Record<string, unknown> = {}) => {
        const object = named(kind, extra);
        return { ...object, destroy: () => { log.push(`destroy ${object.name}`); } };
    };
    const pass = (kind: string) => recorder(log, `${kind}.`, {});

    return recorder(log, '', {
        createBuffer: (descriptor: GPUBufferDescriptor) => destroyable('buffer', {
            size: descriptor.size,
            mapAsync: () => Promise.resolve(),
            // Bytes counting from 1
            getMappedRange: () => Uint8Array.from({ length: descriptor.size }, (_, index) => index + 1).buffer,
        }),
        createTexture: (descriptor: GPUTextureDescriptor) => {
            const size = Array.isArray(descriptor.size) ? descriptor.size : [0, 0];
            const texture = destroyable('texture', {
                format: descriptor.format, width: size[0], height: size[1], sampleCount: descriptor.sampleCount ?? 1,
            });
            return { ...texture, createView: () => ({ name: `view of ${texture.name}` }) };
        },
        createBindGroupLayout: () => named('layout'),
        createPipelineLayout: () => named('pipelines'),
        createSampler: () => named('sampler'),
        createShaderModule: () => named('module', { getCompilationInfo: () => Promise.resolve({ messages: [] }) }),
        createRenderPipeline: () => {
            const pipeline = named('pipeline');
            return { ...pipeline, getBindGroupLayout: (index: number) => ({ name: `layout ${index} of ${pipeline.name}` }) };
        },
        createComputePipeline: () => {
            const pipeline = named('compute');
            return { ...pipeline, getBindGroupLayout: (index: number) => ({ name: `layout ${index} of ${pipeline.name}` }) };
        },
        createBindGroup: () => named('group'),
        createCommandEncoder: () => recorder(log, 'encoder.', {
            beginRenderPass: () => pass('render'),
            beginComputePass: () => pass('compute'),
            finish: () => named('commands'),
        }),
        destroy: () => {},
    }, { queue: recorder(log, 'queue.', {}) }) as unknown as Device & { destroy: () => void };
};

// The canvas, its texture now, 2 x 1, its depth texture and its multisampled one, and what is
// done beyond WebGPU's calls, written down.
const canvasOf = (log: Array<string>): Canvas => ({
    format: 'bgra8unorm',
    texture: () => ({
        name: 'canvas', format: 'bgra8unorm', width: 2, height: 1, sampleCount: 1,
        createView: () => ({ name: 'view of canvas' }),
    }) as unknown as GPUTexture,
    depth: (format, samples) => {
        log.push(`depth(${format} ${samples})`);
        return { name: 'depth', format, createView: () => ({ name: 'view of depth' }) } as unknown as GPUTexture;
    },
    multisampled: (samples) => {
        log.push(`multisampled(${samples})`);
        return { name: 'multisampled', sampleCount: samples, createView: () => ({ name: 'view of multisampled' }) } as unknown as GPUTexture;
    },
    mipmaps: (texture) => {
        log.push(`mipmaps(${shown(texture)})`);
    },
    load: (texture, url, options) => {
        log.push(`load(${texture},${url},${shown(options)})`);
    },
    tell: (event) => {
        log.push(`tell(${shown(event)})`);
    },
});

// `fixture()` in webgpu.rs: one call of each.
const FIXTURE = new Uint8Array([
    1, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 2, 1, 0, 0, 0, 8, 0, 0, 0, 40, 0, 0, 0, 4, 1, 0,
    0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 192, 63, 0, 0, 0, 192, 4, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0,
    0, 7, 0, 0, 0, 5, 2, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 6, 0, 0, 0, 2, 0, 0, 0, 1, 7, 2, 0, 0,
    0, 8, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0, 0, 255, 0, 128, 255, 1, 2, 3, 4, 34, 2, 0, 0,
    0, 35, 3, 0, 0, 0, 1, 1, 0, 6, 0, 0, 0, 47, 97, 46, 112, 110, 103, 8, 4, 0, 0, 0, 1, 2, 1, 0,
    1, 10, 5, 0, 0, 0, 5, 0, 0, 0, 119, 103, 115, 197, 130, 29, 6, 0, 0, 0, 5, 0, 0, 0, 0, 3, 0, 0,
    1, 0, 0, 0, 4, 2, 0, 2, 0, 0, 0, 2, 6, 0, 3, 0, 0, 0, 2, 3, 0, 4, 0, 0, 0, 2, 8, 1, 31, 7, 0,
    0, 0, 1, 6, 0, 0, 0, 12, 8, 0, 0, 0, 7, 0, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 118, 115, 2, 24, 0, 0,
    0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 12, 12, 0, 0, 0, 1, 0, 0, 0, 64, 0, 0, 0, 1, 1, 3, 16, 0,
    0, 0, 3, 0, 0, 0, 4, 2, 1, 2, 1, 15, 1, 3, 1, 5, 0, 0, 0, 2, 0, 0, 0, 102, 115, 2, 0, 0, 15, 9,
    1, 4, 5, 0, 1, 5, 0, 7, 4, 12, 9, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 118, 115, 0, 3,
    0, 0, 0, 0, 0, 1, 13, 10, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 2, 0, 0, 0, 99, 115, 15, 11, 0, 0,
    0, 6, 0, 0, 0, 4, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
    0, 1, 0, 0, 64, 0, 0, 0, 2, 0, 0, 0, 1, 3, 0, 0, 0, 3, 0, 0, 0, 2, 4, 0, 0, 0, 33, 12, 0, 0, 0,
    10, 0, 0, 0, 0, 0, 0, 0, 15, 13, 0, 0, 0, 12, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 17, 2, 2, 4, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 208, 63, 0, 0,
    0, 0, 0, 0, 224, 63, 0, 0, 0, 0, 0, 0, 240, 63, 1, 1, 2, 0, 0, 0, 0, 0, 1, 1, 0, 15, 1, 0, 0,
    128, 63, 0, 20, 8, 0, 0, 0, 21, 1, 0, 0, 0, 11, 0, 0, 0, 22, 1, 0, 0, 0, 1, 0, 0, 0, 8, 0, 0,
    0, 23, 1, 0, 0, 0, 0, 2, 0, 0, 0, 24, 3, 0, 0, 0, 10, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 25, 36,
    0, 0, 0, 144, 1, 0, 0, 6, 0, 0, 0, 255, 255, 255, 255, 2, 0, 0, 0, 27, 0, 0, 0, 0, 0, 0, 128,
    63, 0, 0, 32, 68, 0, 0, 240, 67, 0, 0, 0, 0, 0, 0, 128, 63, 28, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 19, 17, 0, 1, 1, 2, 0, 0, 0, 0, 1, 19, 18, 20, 10, 0, 0, 0, 21, 0, 0, 0, 0,
    13, 0, 0, 0, 26, 4, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 19, 36, 1, 0, 0, 0, 1, 0, 0, 0, 37, 2, 0,
    0, 0, 0, 37, 3, 0, 0, 0, 1, 2, 0, 0, 0, 3, 1, 0, 0, 0, 6, 2, 0, 0, 0, 6, 3, 0, 0, 0, 9, 4, 0,
    0, 0, 11, 5, 0, 0, 0, 30, 6, 0, 0, 0, 32, 7, 0, 0, 0, 14, 8, 0, 0, 0, 14, 9, 0, 0, 0, 14, 10,
    0, 0, 0, 16, 11, 0, 0, 0, 30, 12, 0, 0, 0, 16, 13, 0, 0, 0,
]);

console.log("\n--- Test webgpu: playing the fixture back ---");
{
    const log: Array<string> = [];
    play(recording(log), new Objects(), canvasOf(log), FIXTURE);
    same(log, [
        'createBuffer({"size":8,"usage":40})',
        'queue.writeBuffer(buffer1,4,[0,0,192,63,0,0,0,192])',
        'queue.writeBuffer(buffer1,0,[7,0,0,0])',
        'createTexture({"size":[2,1],"format":"rgba8unorm","usage":6,"mipLevelCount":2,"sampleCount":1})',
        'queue.writeTexture({"texture":"texture2"},[255,0,128,255,1,2,3,4],{"bytesPerRow":8},[2,1])',
        'mipmaps(texture2)',
        'createTexture({"size":[1,1],"format":"rgba8unorm","usage":7})',
        'queue.writeTexture({"texture":"texture3"},[255,255,255,255],{"bytesPerRow":4},[1,1])',
        'load(3,/a.png,{"mipmaps":true,"premultiplied":true,"flip":false})',
        'createSampler({"addressModeU":"repeat","addressModeV":"mirror-repeat","magFilter":"linear","minFilter":"nearest","mipmapFilter":"linear"})',
        'createShaderModule({"code":"wgsł"})',
        'createBindGroupLayout({"entries":[{"binding":0,"visibility":3,"buffer":{"type":"uniform"}},{"binding":1,"visibility":4,"buffer":{"type":"read-only-storage"}},{"binding":2,"visibility":2,"texture":{"sampleType":"float","multisampled":false}},{"binding":3,"visibility":2,"sampler":{"type":"filtering"}},{"binding":4,"visibility":2,"texture":{"sampleType":"depth","multisampled":true}}]})',
        'createPipelineLayout({"bindGroupLayouts":["layout6"]})',
        'createRenderPipeline({"layout":"pipelines7","vertex":{"module":"module5","entryPoint":"vs","buffers":[{"arrayStride":24,"stepMode":"vertex","attributes":[{"format":"float32x3","offset":0,"shaderLocation":0},{"format":"unorm8x4","offset":12,"shaderLocation":1}]},{"arrayStride":64,"stepMode":"instance","attributes":[{"format":"float32x4","offset":16,"shaderLocation":3}]}]},"primitive":{"topology":"triangle-strip","stripIndexFormat":"uint32","frontFace":"cw","cullMode":"back"},"depthStencil":{"format":"depth24plus","depthWriteEnabled":true,"depthCompare":"less-equal"},"fragment":{"module":"module5","entryPoint":"fs","targets":[{"format":"bgra8unorm","writeMask":15},{"format":"rgba16float","blend":{"color":{"srcFactor":"src-alpha","dstFactor":"one-minus-src-alpha","operation":"add"},"alpha":{"srcFactor":"one","dstFactor":"one-minus-src-alpha","operation":"add"}},"writeMask":7}]},"multisample":{"count":4}})',
        'createRenderPipeline({"layout":"auto","vertex":{"module":"module5","entryPoint":"vs","buffers":[]},"primitive":{"topology":"triangle-list","frontFace":"ccw","cullMode":"none"}})',
        'createComputePipeline({"layout":"auto","compute":{"module":"module5","entryPoint":"cs"}})',
        'createBindGroup({"layout":"layout6","entries":[{"binding":0,"resource":{"buffer":"buffer1","offset":0}},{"binding":1,"resource":{"buffer":"buffer1","offset":256,"size":64}},{"binding":2,"resource":"view of texture3"},{"binding":3,"resource":"sampler4"}]})',
        'createBindGroup({"layout":"layout 0 of compute10","entries":[{"binding":0,"resource":{"buffer":"buffer1","offset":0}}]})',
        'createCommandEncoder()',
        'multisampled(4)',
        'depth(depth24plus 4)',
        'encoder.beginRenderPass({"colorAttachments":[{"view":"view of multisampled","resolveTarget":"view of canvas","loadOp":"clear","clearValue":{"r":0,"g":0.25,"b":0.5,"a":1},"storeOp":"discard"},{"view":"view of texture2","loadOp":"load","storeOp":"discard"}],"depthStencilAttachment":{"view":"view of depth","depthLoadOp":"clear","depthClearValue":1,"depthStoreOp":"store"}})',
        'render.setPipeline(pipeline8)',
        'render.setBindGroup(1,group11)',
        'render.setVertexBuffer(1,buffer1,8)',
        'render.setIndexBuffer(buffer1,"uint16",2)',
        'render.draw(3,10,1,0)',
        'render.drawIndexed(36,400,6,-1,2)',
        'render.setViewport(0,1,640,480,0,1)',
        'render.setScissorRect(1,2,3,4)',
        'render.end()',
        'encoder.beginRenderPass({"colorAttachments":[],"depthStencilAttachment":{"view":"view of texture2","depthLoadOp":"load","depthStoreOp":"discard"}})',
        'render.end()',
        'encoder.beginComputePass()',
        'compute.setPipeline(compute10)',
        'compute.setBindGroup(0,group12)',
        'compute.dispatchWorkgroups(4,2,1)',
        'compute.end()',
        'createBuffer({"size":8,"usage":9})',
        'encoder.copyBufferToBuffer(buffer1,0,buffer13,0,8)',
        'createBuffer({"size":256,"usage":9})',
        'encoder.copyTextureToBuffer({"texture":"canvas"},{"buffer":"buffer14","bytesPerRow":256},[2,1])',
        'createBuffer({"size":256,"usage":9})',
        'encoder.copyTextureToBuffer({"texture":"texture2"},{"buffer":"buffer15","bytesPerRow":256},[2,1])',
        'encoder.finish()',
        'queue.submit(["commands16"])',
        'destroy buffer1',
        'destroy texture2',
        'destroy texture3',
    ], 'each call as WebGPU takes it, the passes sent at the end, then what was let go destroyed');

    // The reads, once their copies are mapped: bytes counting from 1, the canvas's as RGBA.
    log.length = 0;
    void (async () => {
        await new Promise((resolve) => setTimeout(resolve, 0));
        same(log, [
            'tell([5,1,[1,2,3,4,5,6,7,8]])',
            'destroy buffer13',
            'tell([6,2,2,1,[3,2,1,4,7,6,5,8]])',
            'destroy buffer14',
            'tell([6,3,2,1,[1,2,3,4,5,6,7,8]])',
            'destroy buffer15',
        ], 'what was read told, rows without their padding, and the copies let go');
    })();
}

const calls = (...ops: Array<Array<number>>): Uint8Array => new Uint8Array(ops.flat());
const u32 = (value: number): Array<number> => [value & 255, (value >> 8) & 255, (value >> 16) & 255, (value >>> 24) & 255];
const buffer = (id: number) => [2, ...u32(id), ...u32(4), ...u32(8)];
const write = (id: number) => [4, ...u32(id), ...u32(0), ...u32(4), 1, 2, 3, 4];
// A pass on the canvas, kept, and with a depth texture as big as it, if `depth`.
const pass = (depth = false) => [17, 1, 0, 0, 0, 0, ...(depth ? [1, 0, 15, 0, 0] : [0])];
const end = [19];

console.log("\n--- Test webgpu: writes and passes in order ---");
{
    const log: Array<string> = [];
    const device = recording(log);
    const objects = new Objects();
    play(device, objects, canvasOf(log), calls(buffer(1), write(1), pass(), end, write(1), pass(), end));
    same(log.filter((call) => !call.startsWith('createBuffer')), [
        'queue.writeBuffer(buffer1,0,[1,2,3,4])',
        'createCommandEncoder()',
        'encoder.beginRenderPass({"colorAttachments":[{"view":"view of canvas","loadOp":"load","storeOp":"store"}]})',
        'render.end()',
        'encoder.finish()',
        'queue.submit(["commands2"])',
        'queue.writeBuffer(buffer1,0,[1,2,3,4])',
        'createCommandEncoder()',
        'encoder.beginRenderPass({"colorAttachments":[{"view":"view of canvas","loadOp":"load","storeOp":"store"}]})',
        'render.end()',
        'encoder.finish()',
        'queue.submit(["commands3"])',
    ], 'a write after a pass sends the pass first, so the pass sees what was there before');

    log.length = 0;
    play(device, objects, canvasOf(log), calls(pass(), write(1), end, [3, ...u32(1)]));
    same(log, [
        'createCommandEncoder()',
        'encoder.beginRenderPass({"colorAttachments":[{"view":"view of canvas","loadOp":"load","storeOp":"store"}]})',
        'queue.writeBuffer(buffer1,0,[1,2,3,4])',
        'render.end()',
        'encoder.finish()',
        'queue.submit(["commands4"])',
        'destroy buffer1',
    ], 'one during a pass goes before it, and a buffer let go is destroyed after the batch using it');

    let thrown = '';
    try {
        play(device, objects, canvasOf(log), calls(write(1)));
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('no buffer 1'), 'a buffer let go is no more');

    thrown = '';
    try {
        play(device, objects, canvasOf(log), calls([200]));
    } catch (error) {
        thrown = String(error);
    }
    assert(thrown.includes('unknown operation 200'), 'an unknown operation is an error');
}

console.log("\n--- Test webgpu: a texture becoming another ---");
{
    const log: Array<string> = [];
    const device = recording(log);
    const objects = new Objects();
    const source = (text: string) => [...u32(text.length), ...Array.from(text, (char) => char.charCodeAt(0))];
    // A layout of a texture and a sampler, a texture from a URL, a sampler, a group with both and
    // one with the sampler only.
    play(device, objects, canvasOf(log), calls(
        [29, ...u32(1), 2, ...u32(0), 2, 6, 0, ...u32(1), 2, 3, 0],
        [35, ...u32(2), 0, 0, 0, ...source('/a.png')],
        [8, ...u32(3), 0, 0, 0, 0, 0],
        [15, ...u32(4), ...u32(1), 2, ...u32(0), 1, ...u32(2), ...u32(1), 2, ...u32(3)],
        [15, ...u32(5), ...u32(1), 1, ...u32(1), 2, ...u32(3)],
    ));
    assert(log.includes('load(2,/a.png,{"mipmaps":false,"premultiplied":false,"flip":false})'), 'the image asked for');

    log.length = 0;
    const image = device.createTexture({ size: [64, 32], format: 'rgba8unorm', usage: 4 });
    objects.replaceTexture(device, 2, image);
    same(log.slice(1), [
        'createBindGroup({"layout":"layout1","entries":[{"binding":0,"resource":"view of texture6"},{"binding":1,"resource":"sampler3"}]})',
    ], 'the bind group with it made again, with the image, and not the other');
    assert(objects.textures.get(2) === image, 'the texture of the id is the image\'s');

    play(device, objects, canvasOf(log), calls([9, ...u32(3)]));
    log.length = 0;
    objects.replaceTexture(device, 2, image);
    assert(log.some((call) => call.includes('"resource":"sampler3"')), 'made again also once its sampler was let go');

    play(device, objects, canvasOf(log), calls([16, ...u32(4)]));
    log.length = 0;
    objects.replaceTexture(device, 2, image);
    same(log, [], 'a group let go is not made again');
}

console.log("\n--- Test webgpu: devices of canvases ---");
{
    // On the page, 160 x 100 CSS pixels.
    class FakeCanvas {
        public width = 300;
        public height = 150;
        public isConnected = true;
        public readonly clientWidth = 160;
        public readonly clientHeight = 100;
        public readonly log: Array<string> = [];
        public other: unknown = null;
        getContext(type: string) {
            this.log.push(`getContext(${type})`);
            if (this.other !== null) {
                return this.other;
            }
            return {
                configure: (configuration: GPUCanvasConfiguration) => {
                    this.log.push(`configure(${configuration.format} ${String(configuration.alphaMode)})`);
                },
                getCurrentTexture: () => ({
                    name: `canvas ${this.width}x${this.height}`, sampleCount: 1, createView: () => ({ name: 'view of canvas' }),
                }),
            };
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

    // A browser giving out devices when `give` is called, each to be lost with `lose`.
    const adapters: Array<string> = [];
    const asked: Array<(device: unknown) => void> = [];
    const devices: Array<{ log: Array<string>, lose: (message: string) => void, destroyed: () => boolean }> = [];
    let hasGpu = true;
    const gpu = {
        getPreferredCanvasFormat: () => 'rgba8unorm',
        requestAdapter: (options: GPURequestAdapterOptions) => {
            adapters.push(shown(options));
            return Promise.resolve({ requestDevice: () => new Promise((resolve) => { asked.push(resolve); }) });
        },
    };
    const give = () => {
        const log: Array<string> = [];
        let lose = (_message: string): void => {};
        let destroyed = false;
        const device = Object.assign(recording(log), {
            lost: new Promise((resolve) => { lose = (message) => resolve({ reason: 'unknown', message }); }),
            destroy: () => { destroyed = true; },
            onuncapturederror: null,
        });
        devices.push({ log, lose: (message) => lose(message), destroyed: () => destroyed });
        asked.shift()?.(device);
    };
    const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

    const told: Array<string> = [];
    const webgpus = new WebGpus((callback, event) => { told.push(`${callback}: ${event.join(',')}`); },
        () => (hasGpu ? gpu as unknown as GPU : undefined));
    const run = (canvas: FakeCanvas, bytes: Uint8Array) => webgpus.run(canvas as unknown as Element, bytes);
    const resized = (canvas: FakeCanvas, width: number, height: number) =>
        watchers[0]?.([{ target: canvas, contentRect: { width, height } }]);
    // The first call of a context: its options, its callback and the scale of its bitmap, 1.
    const context = (alpha: number, power: number, callback: number) =>
        [1, alpha, power, ...u32(callback), ...u32(0), 0, 0, 128, 63];

    void (async () => {
        const canvas = new FakeCanvas();
        const first = calls(context(1, 2, 7), buffer(1));
        run(canvas, first);
        // A view into wasm memory, which the next batch overwrites.
        first.fill(0);
        run(canvas, calls(write(1)));
        await settled();
        same(canvas.log, [], 'the calls wait for the device, the canvas without a context meanwhile');

        give();
        await settled();
        const device = devices[0]!;
        same(canvas.log, ['getContext(webgpu)', 'configure(rgba8unorm premultiplied)'],
            'then the canvas gets its context, in the format the browser prefers, see-through as asked');
        same(adapters, ['{"powerPreference":"high-performance"}'], 'of the GPU asked for');
        same(device.log, [
            'createBuffer({"size":4,"usage":8})',
            'queue.writeBuffer(buffer1,0,[1,2,3,4])',
        ], 'and the calls that waited are played, in order');
        assert(canvas.width === 320 && canvas.height === 200, 'on a bitmap as big as the canvas in device pixels');

        resized(canvas, 160, 100);
        same(told, ['7: 0,320,200'], 'wasm is told the size once the canvas is seen');

        device.log.length = 0;
        run(canvas, calls(pass(true), end, pass(true), end));
        const depths = () => device.log.filter((call) => call.startsWith('createTexture'));
        same(depths(), ['createTexture({"size":[320,200],"format":"depth24plus","sampleCount":1,"usage":16})'],
            'a depth texture as big as the canvas, made once');
        resized(canvas, 200, 100);
        device.log.length = 0;
        run(canvas, calls(pass(true), end));
        same(depths(), ['createTexture({"size":[400,200],"format":"depth24plus","sampleCount":1,"usage":16})'],
            'and again when the canvas is resized');
        assert(device.log.includes('destroy texture2'), 'the old one destroyed');

        // Lost, and another device given.
        device.lose('the GPU was reset');
        const warn = console.warn;
        const warnings: Array<unknown> = [];
        console.warn = (message: unknown) => { warnings.push(message); };
        await settled();
        console.warn = warn;
        assert(warnings.length === 1, 'told in the console as a warning');
        same(adapters.slice(1), ['{"powerPreference":"high-performance"}'], 'a device lost is asked for again');
        run(canvas, calls(write(1)));
        give();
        await settled();
        const second = devices[1]!;
        same(told.slice(2), ['7: 1'], 'wasm is told when another comes');
        assert(!second.log.some((call) => call.startsWith('queue.writeBuffer')), 'the calls meanwhile went nowhere');
        let thrown = '';
        try {
            run(canvas, calls(write(1)));
        } catch (exception) {
            thrown = String(exception);
        }
        assert(thrown.includes('no buffer 1'), 'and the objects made before are gone');
        assert(canvas.log.filter((call) => call.startsWith('getContext')).length === 1, 'the canvas keeps its context');

        webgpus.forget(canvas as unknown as Element);
        assert(second.destroyed(), 'a canvas forgotten lets its device go');
        second.lose('destroyed');
        await settled();
        assert(asked.length === 0, 'and asks for no other');

        // A browser without WebGPU.
        hasGpu = false;
        const none = new FakeCanvas();
        const told_console: Array<unknown> = [];
        console.warn = (message: unknown) => { told_console.push(message); };
        try {
            run(none, calls(context(0, 0, 8), buffer(1)));
            run(none, calls(write(1)));
        } finally {
            console.warn = warn;
        }
        same(told.slice(3), ['8: 2'], 'wasm is told there is no WebGPU');
        assert(told_console.length === 1, 'and the console, once, as a warning: the page draws another way');
        same(none.log, [], 'the canvas free for another context');

        // A canvas with another context already.
        hasGpu = true;
        const taken = new FakeCanvas();
        taken.other = { getParameter: () => 0 };
        console.warn = () => {};
        run(taken, calls(context(0, 0, 9)));
        await settled();
        give();
        await settled();
        console.warn = warn;
        same(told.slice(4), ['9: 2'], 'nor for a canvas with another context');
        assert(devices[2]!.destroyed(), 'whose device is let go');
    // A failed assertion rejects, which ends node with an error.
    })();
}
