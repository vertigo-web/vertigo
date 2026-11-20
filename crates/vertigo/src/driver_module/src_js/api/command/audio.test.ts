// Sound in the browser: sounds decoded and played over each other, music, the volume of all of
// it, and nothing heard until a click, a key or a touch lets the page make sound.

import { Sounds } from "./audio";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const same = (got: unknown, want: unknown, message: string) => {
    const equal = JSON.stringify(got) === JSON.stringify(want);
    if (!equal) {
        console.error('got ', JSON.stringify(got));
        console.error('want', JSON.stringify(want));
    }
    assert(equal, message);
};

// Lets the promises settled so far, and the microtasks, run.
const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

/// What the fakes were asked, in order.
let log: Array<string> = [];

class FakeNode {
    to: unknown = null;
    connect<T>(node: T): T {
        this.to = node;
        return node;
    }
    disconnect() { log.push('disconnect'); }
}

class FakeGain extends FakeNode {
    gain = { value: 1 };
}

class FakeBufferSource extends FakeNode {
    buffer: unknown = null;
    start() { log.push(`start ${String(this.buffer)} at ${(this.to as FakeGain).gain.value}`); }
}

class FakeContext {
    static runningAtFirst = false;
    state = FakeContext.runningAtFirst ? 'running' : 'suspended';
    destination = new FakeNode();
    private changed: Array<() => void> = [];
    addEventListener(_name: string, listener: () => void) { this.changed.push(listener); }
    createGain() { return new FakeGain(); }
    createBufferSource() { return new FakeBufferSource(); }
    createMediaElementSource(element: FakeAudioElement) { log.push(`source of ${element.src}`); return new FakeNode(); }
    decodeAudioData(data: string) { return Promise.resolve(`decoded ${data}`); }
    resume() {
        log.push('resume');
        this.state = 'running';
        this.changed.forEach((listener) => { listener(); });
        return Promise.resolve();
    }
}

class FakeAudioElement {
    preload = '';
    src = '';
    loop = false;
    paused = true;
    currentTime = 5;
    error = null;
    listeners: Record<string, () => void> = {};
    addEventListener(name: string, listener: () => void) { this.listeners[name] = listener; }
    play() {
        log.push(`play ${this.src}`);
        this.paused = false;
        return Promise.resolve();
    }
    pause() { this.paused = true; }
    removeAttribute(name: string) { if (name === 'src') { this.src = ''; } }
    load() { log.push('load'); }
}

/// The gestures the chunk listens to on the document, by their names.
let gestures: Record<string, () => void> = {};
const elements: Array<FakeAudioElement> = [];
const warned: Array<string> = [];

const globals = globalThis as unknown as Record<string, unknown>;
globals['AudioContext'] = FakeContext;
globals['document'] = {
    addEventListener: (name: string, listener: () => void) => { gestures[name] = listener; },
    createElement: () => {
        const element = new FakeAudioElement();
        elements.push(element);
        return element;
    },
};
globals['fetch'] = (url: string) => Promise.resolve({
    ok: url !== '/missing.ogg',
    status: 404,
    arrayBuffer: () => Promise.resolve(`bytes of ${url}`),
});
console.warn = (message: string) => { warned.push(message); };

/// Sound with what wasm is told, the callback 9 listening.
const sounds = () => {
    log = [];
    gestures = {};
    const told: Array<Array<number>> = [];
    const audio = new Sounds((callback, event) => {
        assert(callback === 9, "wasm is told through the callback of `Listen`");
        told.push(event);
    });
    audio.run({ Listen: { callback: 9 } });
    return { audio, told };
};

const masterOf = (audio: Sounds) => (audio as unknown as { master: FakeGain }).master;

async function testSound() {
    console.log("\n--- Test audio 1: a sound, heard only once the page may make sound ---");
    const { audio, told } = sounds();
    audio.run({ LoadSound: { id: 1, url: '/click.ogg' } });
    await settled();
    audio.run({ PlaySound: { id: 1, volume: 1 } });
    await settled();
    same(log, [], "a sound played before the page may make sound isn't heard, even later");
    same(told, [], "and wasm isn't told sound goes through");

    same(
        Object.keys(gestures),
        ['pointerdown', 'pointerup', 'keydown', 'touchend', 'click'],
        "it waits for a click, a key or a touch on the page",
    );
    gestures['keydown']?.();
    await settled();
    same(log, ['resume'], "which lets the context run");
    same(told, [[0]], "and wasm is told sound goes through");

    log = [];
    audio.run({ PlaySound: { id: 1, volume: 0.5 } });
    audio.run({ PlaySound: { id: 1, volume: 1 } });
    same(
        log,
        ['start decoded bytes of /click.ogg at 0.5', 'start decoded bytes of /click.ogg at 1'],
        "then the sound plays when asked, over itself, at the volume asked",
    );
}

async function testMusic() {
    console.log("\n--- Test audio 2: music, started by the gesture that lets the page make sound ---");
    const { audio, told } = sounds();
    audio.run({ LoadMusic: { id: 2, url: '/theme.ogg' } });
    const element = elements[elements.length - 1];
    assert(element !== undefined && element.preload === 'auto', "the music streams from an <audio>");
    same(log, ['source of /theme.ogg'], "through Web Audio, so its volume works everywhere");

    audio.run({ PlayMusic: { id: 2 } });
    assert(element?.paused === true, "it doesn't play before the page may make sound");
    log = [];
    gestures['click']?.();
    same(log, ['resume', 'play /theme.ogg'], "the gesture lets the context run and starts it, in the gesture");

    audio.run({ LoopMusic: { id: 2, looping: true } });
    assert(element?.loop === true, "it loops when asked");
    audio.run({ StopMusic: { id: 2 } });
    assert(element?.paused === true && element.currentTime === 0, "a stop pauses it and goes back to its start");

    element?.listeners['ended']?.();
    await settled();
    same(told, [[0], [1, 2]], "wasm is told of its end, by its id");

    log = [];
    audio.run({ Forget: { id: 2 } });
    assert(element?.src === '', "a music forgotten lets its file go");
    same(log, ['load', 'disconnect', 'disconnect'], "and its nodes");
}

function testVolumes() {
    console.log("\n--- Test audio 3: the volume of all sound, of a music, and muting ---");
    const { audio } = sounds();
    audio.run({ LoadMusic: { id: 3, url: '/quiet.ogg' } });
    audio.run({ MusicVolume: { id: 3, volume: 0.25 } });
    const gains = (audio as unknown as { musics: Map<number, { gain: FakeGain }> }).musics;
    assert(gains.get(3)?.gain.gain.value === 0.25, "a music has a volume of its own");

    audio.run({ Volume: { volume: 0.5 } });
    assert(masterOf(audio).gain.value === 0.5, "all sound goes through one volume");
    audio.run({ Muted: { muted: true } });
    assert(masterOf(audio).gain.value === 0, "which muting silences");
    audio.run({ Volume: { volume: 0.8 } });
    assert(masterOf(audio).gain.value === 0, "and keeps silent when the volume changes");
    audio.run({ Muted: { muted: false } });
    assert(masterOf(audio).gain.value === 0.8, "until it lets sound be heard, at the volume it has then");
}

async function testFailures() {
    console.log("\n--- Test audio 4: a sound that can't be played, and one forgotten ---");
    const { audio } = sounds();
    gestures['pointerdown']?.();
    audio.run({ LoadSound: { id: 4, url: '/missing.ogg' } });
    audio.run({ LoadSound: { id: 5, url: '/gone.ogg' } });
    audio.run({ Forget: { id: 5 } });
    await settled();
    assert(warned.some((message) => message.includes('/missing.ogg')), "a sound that doesn't come says so in the console");
    log = [];
    audio.run({ PlaySound: { id: 4, volume: 1 } });
    audio.run({ PlaySound: { id: 5, volume: 1 } });
    same(log, [], "and neither it nor one forgotten while it came plays");
}

async function testAllowedAtOnce() {
    console.log("\n--- Test audio 5: a browser which lets the page make sound at once ---");
    FakeContext.runningAtFirst = true;
    const { audio, told } = sounds();
    await settled();
    same(told, [[0]], "wasm is told sound goes through, after the call it came in");

    audio.run({ LoadMusic: { id: 6, url: '/at-once.ogg' } });
    log = [];
    audio.run({ PlayMusic: { id: 6 } });
    same(log, ['play /at-once.ogg'], "and music plays when asked");
    FakeContext.runningAtFirst = false;
}

async function main() {
    await testSound();
    await testMusic();
    testVolumes();
    await testFailures();
    await testAllowedAtOnce();
}

void main();
