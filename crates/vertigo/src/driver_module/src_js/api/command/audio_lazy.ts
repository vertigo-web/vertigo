// Sound, in a chunk of its own loaded the first time a page makes a sound or music, so that a page
// without them never downloads the code. The chunk is `audio.ts`, see `canvas_lazy.ts` for how
// chunks are made. The calls coming before it are kept in order, as the later ones play what the
// earlier ones load, and run when it is in.

import type { AudioOp, Sounds, Tell } from "./audio";

// What `Api` asks of sound.
export type AudioApi = Pick<Sounds, 'run'>;

export class LazyAudio {
    private audio: AudioApi | null = null;
    private loading = false;
    private readonly calls: Array<AudioOp> = [];

    private readonly load: () => Promise<AudioApi>;

    // `tell` is how sound tells wasm of itself, see `Sounds`.
    public constructor(tell: Tell, load?: () => Promise<AudioApi>) {
        this.load = load ?? (async () => {
            const audio = await import("./audio");
            return new audio.Sounds(tell);
        });
    }

    public run(op: AudioOp) {
        if (this.audio !== null) {
            this.audio.run(op);
            return;
        }

        this.calls.push(op);
        this.start();
    }

    // Once. A chunk that failed to load is asked for again by the next calls.
    private start() {
        if (this.loading) {
            return;
        }

        this.loading = true;
        this.load().then(
            (audio) => {
                this.audio = audio;
                for (const op of this.calls) {
                    try {
                        audio.run(op);
                    } catch (error) {
                        console.error(error);
                    }
                }
                this.calls.length = 0;
            },
            (error: unknown) => {
                this.loading = false;
                console.error('audio: the code for sound failed to load', error);
            },
        );
    }
}
