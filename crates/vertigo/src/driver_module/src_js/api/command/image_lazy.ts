// Pictures made smaller, in a chunk of its own loaded with the first one, so that a page that
// resizes none never downloads the code. The chunk is `image.ts`, see `canvas_lazy.ts` for how
// chunks are made. The calls coming before it wait for it.

import type { ImageOp, Images, Tell } from "./image";

// What `Api` asks of pictures.
export type ImagesApi = Pick<Images, 'run'>;

export class LazyImages {
    private images: ImagesApi | null = null;
    private loading = false;
    private readonly calls: Array<ImageOp> = [];

    private readonly load: () => Promise<ImagesApi>;

    // `tell` is how the answer to a call gets to wasm, see `Images`.
    public constructor(private readonly tell: Tell, load?: () => Promise<ImagesApi>) {
        this.load = load ?? (async () => {
            const image = await import("./image");
            return new image.Images(tell);
        });
    }

    public run(op: ImageOp) {
        if (this.images !== null) {
            this.images.run(op);
            return;
        }

        this.calls.push(op);
        this.start();
    }

    // Once. Each call is waited for in wasm, so when the chunk fails to load the ones waiting
    // are told, and the next call asks for the chunk again.
    private start() {
        if (this.loading) {
            return;
        }

        this.loading = true;
        this.load().then(
            (images) => {
                this.images = images;
                for (const op of this.calls.splice(0)) {
                    images.run(op);
                }
            },
            (error: unknown) => {
                this.loading = false;
                console.error('image: the code for pictures failed to load', error);
                for (const op of this.calls.splice(0)) {
                    this.tell(op.Resize.callback, { Failed: { message: 'the code for pictures failed to load' } });
                }
            },
        );
    }
}
