// Canvases, in a chunk of their own loaded the first time a page needs it, so that a page
// without canvases never downloads the code drawing on them.
//
// The chunk is `canvas.ts`, and nothing else: rollup makes a chunk of what is imported
// dynamically, and `rollup.config.mjs` fails the build when it imports anything from
// `wasm_run.js`, which vertigo-cli renames. It is fetched when the first drawing comes, or
// earlier when the server sent the page with a canvas, see `DriverDom`. Drawings coming before
// it are kept, the last one of each canvas, and drawn when it is in.

import type { Canvases } from "./canvas";

// What `DriverDom` asks of canvases.
export type CanvasesApi = Pick<Canvases, 'draw' | 'paintDrawn' | 'preload' | 'forget'>;

export class LazyCanvases {
    private canvases: CanvasesApi | null = null;
    private loading = false;
    // What came before the chunk, see `loaded`.
    private readonly drawings: Map<HTMLCanvasElement, Uint8Array> = new Map();
    private readonly images: Set<string> = new Set();

    public constructor(
        private readonly load: () => Promise<CanvasesApi> =
            async () => new (await import("./canvas")).Canvases(),
    ) {}

    // Starts loading the chunk, before a drawing comes.
    public prefetch() {
        this.start();
    }

    // `bytes` is good only for the call it came in (see `DriverDom.update`), so it is copied.
    public draw(node: Element, bytes: Uint8Array) {
        if (this.canvases !== null) {
            this.canvases.draw(node, bytes);
            return;
        }

        if (!(node instanceof HTMLCanvasElement)) {
            throw new Error('draw: the element is not a canvas');
        }

        this.drawings.set(node, bytes.slice());
        this.start();
    }

    // A drawing waiting for the chunk is drawn when it is in, see `loaded`.
    public paintDrawn() {
        this.canvases?.paintDrawn();
    }

    public preload(url: string) {
        if (this.canvases !== null) {
            this.canvases.preload(url);
            return;
        }

        this.images.add(url);
        this.start();
    }

    public forget(node: Element | Comment | Text) {
        if (this.canvases !== null) {
            this.canvases.forget(node);
        } else if (node instanceof HTMLCanvasElement) {
            this.drawings.delete(node);
        }
    }

    // Once. A chunk that failed to load is asked for again by the next drawing.
    private start() {
        if (this.loading || this.canvases !== null) {
            return;
        }

        this.loading = true;
        this.load().then(
            (canvases) => this.loaded(canvases),
            (error: unknown) => {
                this.loading = false;
                console.error('canvas: the code drawing on canvases failed to load', error);
            },
        );
    }

    // Draws what waited for the chunk, out of any batch of DOM changes, so with the styles in
    // place, as `Canvases.paintDrawn` needs.
    private loaded(canvases: CanvasesApi) {
        this.canvases = canvases;

        for (const url of this.images) {
            canvases.preload(url);
        }
        for (const [canvas, drawing] of this.drawings) {
            canvases.draw(canvas, drawing);
        }
        this.images.clear();
        this.drawings.clear();

        canvases.paintDrawn();
    }
}
