// WebGL, in a chunk of their own loaded the first time a page draws with it, so that a page
// without it never downloads the code. The chunk is `webgl.ts`, see `canvas_lazy.ts` for how
// chunks are made. The calls coming before it are kept, all of them in order, as they make the
// objects the later ones use, and played when it is in.

import type { Tell, WebGls } from "./webgl";

// What `DriverDom` asks of WebGL.
export type WebGlsApi = Pick<WebGls, 'run' | 'forget'>;

export class LazyWebGls {
    private webgls: WebGlsApi | null = null;
    private loading = false;
    // The calls that came before the chunk, of each canvas in order.
    private readonly calls: Map<HTMLCanvasElement, Array<Uint8Array>> = new Map();

    private readonly load: () => Promise<WebGlsApi>;

    // `tell` is how the canvases tell wasm of themselves, see `WebGls`.
    public constructor(tell: Tell, load?: () => Promise<WebGlsApi>) {
        this.load = load ?? (async () => new (await import("./webgl")).WebGls(tell));
    }

    // `bytes` is good only for the call it came in (see `DriverDom.update`), so it is copied.
    public run(node: Element, bytes: Uint8Array) {
        if (this.webgls !== null) {
            this.webgls.run(node, bytes);
            return;
        }

        if (!(node instanceof HTMLCanvasElement)) {
            throw new Error('webgl: the element is not a canvas');
        }

        const calls = this.calls.get(node) ?? [];
        calls.push(bytes.slice());
        this.calls.set(node, calls);
        this.start();
    }

    public forget(node: Element | Comment | Text) {
        if (this.webgls !== null) {
            this.webgls.forget(node);
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
            (webgls) => {
                this.webgls = webgls;
                for (const [canvas, calls] of this.calls) {
                    for (const bytes of calls) {
                        try {
                            webgls.run(canvas, bytes);
                        } catch (error) {
                            console.error(error);
                        }
                    }
                }
                this.calls.clear();
            },
            (error: unknown) => {
                this.loading = false;
                console.error('webgl: the code for WebGL failed to load', error);
            },
        );
    }
}
