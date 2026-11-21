// WebGL and WebGPU, in a chunk of their own loaded the first time a page draws with either of
// them, so that a page without them never downloads the code. The chunk is `gpu.ts`, see
// `canvas_lazy.ts` for how chunks are made. The calls coming before it are kept, all of them in
// order, as they make the objects the later ones use, and played when it is in.

import type { Tell } from "./canvas_sizes";
import type { WebGls } from "./webgl";

export type Api = 'webgl' | 'webgpu';

// What `DriverDom` asks of WebGL, and of WebGPU.
export type ContextsApi = Pick<WebGls, 'run' | 'forget'>;
export type GpusApi = Record<Api, ContextsApi>;

export class LazyGpus {
    private gpus: GpusApi | null = null;
    private loading = false;
    // The calls that came before the chunk, of each canvas in order.
    private readonly calls: Map<HTMLCanvasElement, Array<[Api, Uint8Array]>> = new Map();

    private readonly load: () => Promise<GpusApi>;

    // `tell` is how the canvases tell wasm of themselves, see `CanvasSizes`.
    public constructor(tell: Tell, load?: () => Promise<GpusApi>) {
        this.load = load ?? (async () => {
            const gpu = await import("./gpu");
            return { webgl: new gpu.WebGls(tell), webgpu: new gpu.WebGpus(tell) };
        });
    }

    // `bytes` is good only for the call it came in (see `DriverDom.update`), so it is copied.
    public run(api: Api, node: Element, bytes: Uint8Array) {
        if (this.gpus !== null) {
            this.gpus[api].run(node, bytes);
            return;
        }

        if (!(node instanceof HTMLCanvasElement)) {
            throw new Error(`${api}: the element is not a canvas`);
        }

        const calls = this.calls.get(node) ?? [];
        calls.push([api, bytes.slice()]);
        this.calls.set(node, calls);
        this.start();
    }

    public forget(node: Element | Comment | Text) {
        if (this.gpus !== null) {
            this.gpus.webgl.forget(node);
            this.gpus.webgpu.forget(node);
        } else if (node instanceof HTMLCanvasElement) {
            this.calls.delete(node);
        }
    }

    // Once. A chunk that failed to load is asked for again by the next calls.
    private start() {
        if (this.loading) {
            return;
        }

        this.loading = true;
        this.load().then(
            (gpus) => {
                this.gpus = gpus;
                for (const [canvas, calls] of this.calls) {
                    for (const [api, bytes] of calls) {
                        try {
                            gpus[api].run(canvas, bytes);
                        } catch (error) {
                            console.error(error);
                        }
                    }
                }
                this.calls.clear();
            },
            (error: unknown) => {
                this.loading = false;
                console.error('gpu: the code for WebGL and WebGPU failed to load', error);
            },
        );
    }
}
