// Sound for `vertigo::audio` (`audio.rs`), with Web Audio: sounds decoded once and played over
// each other, music streamed by `<audio>` elements, and the volume of all of it, every one of them
// through one gain, so the volume works where an `<audio>`'s own doesn't (iOS).
//
// A browser lets a page make sound only once the user has clicked, pressed a key or touched it:
// until then the context is suspended. The first of those events asks for it to run, and starts
// the music asked to play meanwhile, both in the event, as some browsers want. Sounds played
// before aren't heard rather than all heard at once then.
//
// A chunk of its own, loaded the first time a page makes a sound (`audio_lazy.ts`).

/// How the browser tells wasm of sound through the callback of `Listen`.
export type Tell = (callback: number, event: Array<number>) => void;

/// A call of `vertigo::audio`, as `AudioOp` in `dev/command.rs` is sent.
export type AudioOp =
    | { Listen: { callback: number } }
    | { LoadSound: { id: number, url: string } }
    | { PlaySound: { id: number, volume: number } }
    | { LoadMusic: { id: number, url: string } }
    | { PlayMusic: { id: number } }
    | { PauseMusic: { id: number } }
    | { StopMusic: { id: number } }
    | { LoopMusic: { id: number, looping: boolean } }
    | { MusicVolume: { id: number, volume: number } }
    | { Volume: { volume: number } }
    | { Muted: { muted: boolean } }
    | { Forget: { id: number } };

/// The events wasm is told, see `event` in `audio.rs`.
const ALLOWED = 0;
const ENDED = 1;

/// The events of the user after which a browser lets a page make sound.
const GESTURES = ['pointerdown', 'pointerup', 'keydown', 'touchend', 'click'];

type Music = {
    element: HTMLAudioElement,
    source: MediaElementAudioSourceNode,
    gain: GainNode,
    /// Asked to play, so it starts when the browser lets it.
    wanted: boolean,
};

export class Sounds {
    private readonly context: AudioContext;
    private readonly master: GainNode;
    private volume = 1;
    private muted = false;
    /// The sounds, `null` until they are decoded.
    private readonly sounds: Map<number, AudioBuffer | null> = new Map();
    private readonly musics: Map<number, Music> = new Map();
    private callback: number | null = null;
    private toldAllowed = false;

    public constructor(private readonly tell: Tell) {
        this.context = new AudioContext();
        this.master = this.context.createGain();
        this.master.connect(this.context.destination);
        this.context.addEventListener('statechange', () => { this.tellAllowed(); });

        const unlock = () => { this.unlock(); };
        GESTURES.forEach((name) => { document.addEventListener(name, unlock, { capture: true, passive: true }); });
    }

    public run(op: AudioOp) {
        if ('Listen' in op) {
            this.callback = op.Listen.callback;
            this.tellAllowed();
        } else if ('LoadSound' in op) {
            this.loadSound(op.LoadSound.id, op.LoadSound.url);
        } else if ('PlaySound' in op) {
            this.playSound(op.PlaySound.id, op.PlaySound.volume);
        } else if ('LoadMusic' in op) {
            this.loadMusic(op.LoadMusic.id, op.LoadMusic.url);
        } else if ('PlayMusic' in op) {
            this.playMusic(op.PlayMusic.id);
        } else if ('PauseMusic' in op) {
            this.withMusic(op.PauseMusic.id, (music) => {
                music.wanted = false;
                music.element.pause();
            });
        } else if ('StopMusic' in op) {
            this.withMusic(op.StopMusic.id, (music) => {
                music.wanted = false;
                music.element.pause();
                music.element.currentTime = 0;
            });
        } else if ('LoopMusic' in op) {
            this.withMusic(op.LoopMusic.id, (music) => { music.element.loop = op.LoopMusic.looping; });
        } else if ('MusicVolume' in op) {
            this.withMusic(op.MusicVolume.id, (music) => { music.gain.gain.value = op.MusicVolume.volume; });
        } else if ('Volume' in op) {
            this.volume = op.Volume.volume;
            this.master.gain.value = this.muted ? 0 : this.volume;
        } else if ('Muted' in op) {
            this.muted = op.Muted.muted;
            this.master.gain.value = this.muted ? 0 : this.volume;
        } else {
            this.forget(op.Forget.id);
        }
    }

    private loadSound(id: number, url: string) {
        this.sounds.set(id, null);
        fetch(url)
            .then((response) => {
                if (!response.ok) {
                    throw new Error(`${response.status}`);
                }
                return response.arrayBuffer();
            })
            .then((data) => this.context.decodeAudioData(data))
            .then(
                (buffer) => {
                    // Unless forgotten meanwhile
                    if (this.sounds.has(id)) {
                        this.sounds.set(id, buffer);
                    }
                },
                (error: unknown) => { console.warn(`audio: the sound ${url} can't be played`, error); },
            );
    }

    private playSound(id: number, volume: number) {
        const buffer = this.sounds.get(id);
        if (buffer === undefined || buffer === null || this.context.state !== 'running') {
            return;
        }

        const source = this.context.createBufferSource();
        source.buffer = buffer;
        const gain = this.context.createGain();
        gain.gain.value = volume;
        source.connect(gain).connect(this.master);
        source.start();
    }

    private loadMusic(id: number, url: string) {
        const element = document.createElement('audio');
        element.preload = 'auto';
        element.src = url;
        const source = this.context.createMediaElementSource(element);
        const gain = this.context.createGain();
        source.connect(gain).connect(this.master);
        const music = { element, source, gain, wanted: false };
        this.musics.set(id, music);

        element.addEventListener('ended', () => {
            music.wanted = false;
            this.send([ENDED, id]);
        });
        element.addEventListener('error', () => {
            console.warn(`audio: the music ${url} can't be played`, element.error);
        });
    }

    private playMusic(id: number) {
        this.withMusic(id, (music) => {
            music.wanted = true;
            if (this.context.state === 'running') {
                this.start(music);
            }
        });
    }

    private start(music: Music) {
        music.element.play().catch((error: unknown) => {
            // Paused before it started, f. ex. by a stop, isn't a refusal.
            if (!(error instanceof DOMException && error.name === 'AbortError')) {
                console.warn('audio: the music can\'t play yet', error);
            }
        });
    }

    private withMusic(id: number, action: (music: Music) => void) {
        const music = this.musics.get(id);
        if (music === undefined) {
            console.error(`audio: no music ${id}`);
            return;
        }
        action(music);
    }

    private forget(id: number) {
        this.sounds.delete(id);

        const music = this.musics.get(id);
        if (music !== undefined) {
            this.musics.delete(id);
            music.element.pause();
            music.element.removeAttribute('src');
            music.element.load();
            music.source.disconnect();
            music.gain.disconnect();
        }
    }

    /// In a click, a key or a touch: the context asked to run, and the music waiting for it
    /// started.
    private unlock() {
        if (this.context.state === 'running') {
            return;
        }

        this.context.resume().catch((error: unknown) => { console.warn('audio: no sound yet', error); });
        for (const music of this.musics.values()) {
            if (music.wanted) {
                this.start(music);
            }
        }
    }

    private tellAllowed() {
        if (!this.toldAllowed && this.callback !== null && this.context.state === 'running') {
            this.toldAllowed = true;
            this.send([ALLOWED]);
        }
    }

    /// After the call it may come in, which wasm is in the middle of.
    private send(event: Array<number>) {
        const callback = this.callback;
        if (callback !== null) {
            queueMicrotask(() => { this.tell(callback, event); });
        }
    }
}
