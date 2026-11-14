// The click, wheel and pointer handlers: what a `ClickEvent`, `WheelEvent` or `PointerEvent`
// reply does to the browser's own handling of the event, and what the handlers tell wasm. And
// `scroll`, which tells wasm where an element is scrolled to, `resize`, which tells it how big
// an element is, `change_checked`, which tells it whether a box is checked, `change_file`, which
// sends it the files picked into an input, `media`, which tells it what a video or an audio
// does, and `message`, which passes on the messages from the page in a frame.

import { CallbackManager, click, keepEarlyMessage, pointer, scroll, wheel } from "./callbackManager";
import { MapNodes } from "./map_nodes";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

// --- MOCKS ---
class MockEvent {
    defaultPrevented = false;
    propagationStopped = false;
    preventDefault() { this.defaultPrevented = true; }
    stopPropagation() { this.propagationStopped = true; }
}

/// A click with Ctrl held.
class MockClickEvent extends MockEvent {
    altKey = false;
    ctrlKey = true;
    shiftKey = false;
    metaKey = false;
}

/// The element with the listener: its top left corner is at (100, 200) and it is 300 x 150.
class MockElement {
    getBoundingClientRect() { return { left: 100, top: 200, width: 300, height: 150 }; }
    setPointerCapture(_pointerId: number) {}
    hasPointerCapture(_pointerId: number) { return false; }
    releasePointerCapture(_pointerId: number) {}
}

// What the handlers check the events and their targets against.
(globalThis as any).MouseEvent = MockClickEvent;
(globalThis as any).Element = MockElement;

/// A click whose `on_click` in wasm replied with `reply` - what `ClickEvent` is serialized to.
function clickReplying(reply: unknown): MockClickEvent {
    const event = new MockClickEvent();
    click(event as unknown as Event, () => reply as any);
    return event;
}

// --- TESTS ---

function testDefaultActionIsLeftToTheBrowser() {
    console.log("\n--- Test click 1: the default action happens ---");
    const event = clickReplying({ stop_propagation: false, prevent_default: false });
    assert(!event.defaultPrevented, "a link opens, a form is submitted, a checkbox toggles");
    assert(!event.propagationStopped, "and the click reaches the parents");
}

function testPreventDefault() {
    console.log("\n--- Test click 2: ClickEvent::prevent_default ---");
    const event = clickReplying({ stop_propagation: false, prevent_default: true });
    assert(event.defaultPrevented, "prevents the default action");
    assert(!event.propagationStopped, "without stopping propagation");
}

function testStopPropagation() {
    console.log("\n--- Test click 3: ClickEvent::stop_propagation ---");
    const event = clickReplying({ stop_propagation: true, prevent_default: false });
    assert(event.propagationStopped, "stops propagation");
    assert(!event.defaultPrevented, "without preventing the default action");
}

function testClickPayload() {
    console.log("\n--- Test click 0: what wasm gets ---");
    let sent: unknown;
    click(new MockClickEvent() as unknown as Event, (value) => {
        sent = value;
        return null;
    });
    assert(
        JSON.stringify(sent) === JSON.stringify([false, true, false, false]),
        `the modifier keys: ${JSON.stringify(sent)}`,
    );
}

function testReplyWithoutFlags() {
    console.log("\n--- Test click 4: a reply that isn't the flags ---");
    for (const reply of [undefined, null, [true, true], 'prevent_default']) {
        const event = clickReplying(reply);
        assert(
            !event.defaultPrevented && !event.propagationStopped,
            // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- `JSON.stringify(undefined)` is `undefined`, whatever its type says
            `${JSON.stringify(reply) ?? 'undefined'} changes nothing`,
        );
    }
}

/// A wheel turned 120 px down, with Ctrl held, over (130, 240) of the viewport, on an element
/// whose top left corner is at (100, 200) and which is 300 x 150.
class MockWheelEvent extends MockEvent {
    deltaX = -2.5;
    deltaY = 120;
    deltaZ = 0;
    deltaMode = 0;
    clientX = 130;
    clientY = 240;
    altKey = false;
    ctrlKey = true;
    shiftKey = false;
    metaKey = false;
    currentTarget = new MockElement();
}

(globalThis as any).WheelEvent = MockWheelEvent;

/// A wheel whose `on_wheel` in wasm replied with `reply`, and what wasm got.
function wheelReplying(reply: unknown): { event: MockWheelEvent, sent: unknown } {
    const event = new MockWheelEvent();
    let sent: unknown;
    wheel(event as unknown as Event, (value) => {
        sent = value;
        return reply as any;
    });
    return { event, sent };
}

function testWheelPayload() {
    console.log("\n--- Test wheel 1: what wasm gets ---");
    const { sent } = wheelReplying(null);
    assert(
        JSON.stringify(sent) === JSON.stringify([-2.5, 120, 0, 0, 30, 40, 300, 150, false, true, false, false]),
        `deltas, mode, pointer from the element's corner, its size, modifiers: ${JSON.stringify(sent)}`,
    );
}

function testWheelScrollsByDefault() {
    console.log("\n--- Test wheel 2: the page scrolls ---");
    const { event } = wheelReplying({ stop_propagation: false, prevent_default: false });
    assert(!event.defaultPrevented, "the page scrolls");
    assert(!event.propagationStopped, "and the parents get the wheel");
}

function testWheelPreventDefault() {
    console.log("\n--- Test wheel 3: WheelEvent::prevent_default ---");
    const { event } = wheelReplying({ stop_propagation: false, prevent_default: true });
    assert(event.defaultPrevented, "the page doesn't scroll");
    assert(!event.propagationStopped, "and the parents still get the wheel");
}

function testWheelStopPropagation() {
    console.log("\n--- Test wheel 4: WheelEvent::stop_propagation ---");
    const { event } = wheelReplying({ stop_propagation: true, prevent_default: false });
    assert(event.propagationStopped, "the parents don't get the wheel");
    assert(!event.defaultPrevented, "and the page scrolls");
}

/// A node that keeps how its listeners were added.
class MockNode {
    added: Array<[string, unknown]> = [];
    listeners: Array<(event: Event) => void> = [];
    removed: string[] = [];
    addEventListener(name: string, listener: (event: Event) => void, options: unknown) {
        this.added.push([name, options]);
        this.listeners.push(listener);
    }
    removeEventListener(name: string, _listener: unknown) {
        this.removed.push(name);
    }
}

function testWheelListenerIsNotPassive() {
    console.log("\n--- Test wheel 5: the listener ---");
    const node = new MockNode();
    const nodes = { get: () => node } as unknown as MapNodes;
    const manager = new CallbackManager(() => ({}) as any);

    manager.add(nodes, 1, 'wheel', 10);
    manager.add(nodes, 1, 'click', 11);

    assert(
        JSON.stringify(node.added[0]) === JSON.stringify(['wheel', { passive: false }]),
        "a wheel listener isn't passive, so prevent_default works even on <body>",
    );
    assert(JSON.stringify(node.added[1]) === JSON.stringify(['click', false]), "the others are added as before");
}

/// A finger touching (130, 240) of the viewport, with Shift held, on an element whose top left
/// corner is at (100, 200) and which is 300 x 150. The element keeps what it was asked to
/// capture. The finger touched a child of it, which the browser made capture the finger, and
/// which keeps what it was asked to release.
class MockPointerEvent extends MockEvent {
    pointerId = 7;
    pointerType = 'touch';
    isPrimary = true;
    button = 0;
    buttons = 1;
    clientX = 130;
    clientY = 240;
    altKey = false;
    ctrlKey = false;
    shiftKey = true;
    metaKey = false;
    captured: Array<number> = [];
    currentTarget = Object.assign(new MockElement(), {
        setPointerCapture: (pointerId: number) => { this.captured.push(pointerId); },
    });
    released: Array<number> = [];
    target = Object.assign(new MockElement(), {
        hasPointerCapture: (pointerId: number): boolean => pointerId === 7,
        releasePointerCapture: (pointerId: number) => { this.released.push(pointerId); },
    });
}

(globalThis as any).PointerEvent = MockPointerEvent;

/// A pointer event whose listener in wasm replied with `reply`, and what wasm got.
function pointerReplying(reply: unknown): { event: MockPointerEvent, sent: unknown } {
    const event = new MockPointerEvent();
    let sent: unknown;
    pointer(event as unknown as Event, (value) => {
        sent = value;
        return reply as any;
    });
    return { event, sent };
}

function testPointerPayload() {
    console.log("\n--- Test pointer 1: what wasm gets ---");
    const { sent } = pointerReplying(null);
    assert(
        JSON.stringify(sent) === JSON.stringify([7, 'touch', true, 0, 1, 30, 40, 300, 150, false, false, true, false]),
        `id, type, primary, buttons, pointer from the element's corner, its size, modifiers: ${JSON.stringify(sent)}`,
    );
}

function testPointerByDefault() {
    console.log("\n--- Test pointer 2: nothing asked ---");
    const { event } = pointerReplying({ stop_propagation: false, prevent_default: false, capture_pointer: false, release_pointer: false });
    assert(!event.defaultPrevented && !event.propagationStopped, "the browser goes on as usual");
    assert(event.captured.length === 0 && event.released.length === 0, "and captures nothing, or releases");
}

function testPointerCapture() {
    console.log("\n--- Test pointer 3: PointerEvent::capture_pointer ---");
    const { event } = pointerReplying({ stop_propagation: false, prevent_default: false, capture_pointer: true, release_pointer: false });
    assert(JSON.stringify(event.captured) === JSON.stringify([7]), "the element with the listener captures the pointer");
    assert(!event.defaultPrevented && !event.propagationStopped, "and that's all");
}

function testPointerCaptureOfAPointerThatIsUp() {
    console.log("\n--- Test pointer 3b: capture_pointer of a pointer that's up ---");
    const event = new MockPointerEvent();
    event.currentTarget.setPointerCapture = () => { throw new Error('NotFoundError'); };
    let threw = false;
    try {
        pointer(event as unknown as Event, () => ({ stop_propagation: true, prevent_default: false, capture_pointer: true, release_pointer: false }));
    } catch {
        threw = true;
    }
    assert(!threw, "the browser's error goes to the console, not out of the listener");
    assert(event.propagationStopped, "and the other flags still apply");
}

function testPointerRelease() {
    console.log("\n--- Test pointer 3c: PointerEvent::release_pointer ---");
    const { event } = pointerReplying({ stop_propagation: false, prevent_default: false, capture_pointer: false, release_pointer: true });
    assert(JSON.stringify(event.released) === JSON.stringify([7]), "the element holding the pointer lets it go");
    assert(event.captured.length === 0, "and the element with the listener doesn't take it");
    assert(!event.defaultPrevented && !event.propagationStopped, "and that's all");
}

function testPointerReleaseOfAPointerNotCaptured() {
    console.log("\n--- Test pointer 3d: release_pointer of a pointer nobody captured ---");
    const event = new MockPointerEvent();
    event.target.hasPointerCapture = () => false;
    event.target.releasePointerCapture = () => { throw new Error('NotFoundError'); };
    let threw = false;
    try {
        pointer(event as unknown as Event, () => ({ stop_propagation: true, prevent_default: false, capture_pointer: false, release_pointer: true }));
    } catch {
        threw = true;
    }
    assert(!threw, "nothing to let go of, and no error");
    assert(event.propagationStopped, "and the other flags still apply");
}

function testPointerFlags() {
    console.log("\n--- Test pointer 4: prevent_default and stop_propagation ---");
    const { event } = pointerReplying({ stop_propagation: true, prevent_default: true, capture_pointer: false, release_pointer: false });
    assert(event.defaultPrevented && event.propagationStopped, "both, as for a click");
    assert(event.captured.length === 0, "without capturing");
}

function testPointerListeners() {
    console.log("\n--- Test pointer 5: the listeners ---");
    const node = new MockNode();
    const nodes = { get: () => node } as unknown as MapNodes;
    const manager = new CallbackManager(() => ({}) as any);
    const names = ['pointerdown', 'pointermove', 'pointerup', 'pointercancel'];

    names.forEach((name, index) => manager.add(nodes, 1, name, 20 + index));

    assert(
        JSON.stringify(node.added) === JSON.stringify(names.map((name) => [name, false])),
        "each goes on the DOM event of its name",
    );
}

/// An element whose content is scrolled 120.5 px down, on a zoomed page: 900 px tall, of which
/// 300 px show.
class MockScrollEvent extends MockEvent {
    currentTarget = Object.assign(new MockElement(), {
        scrollTop: 120.5,
        scrollLeft: 0,
        scrollWidth: 400,
        scrollHeight: 900,
        clientWidth: 385,
        clientHeight: 300,
    });
}

function testScrollPayload() {
    console.log("\n--- Test scroll 1: what wasm gets ---");
    const event = new MockScrollEvent();
    let sent: unknown;
    scroll(event as unknown as Event, (value) => {
        sent = value;
        return null;
    });
    assert(
        JSON.stringify(sent) === JSON.stringify([120.5, 0, 400, 900, 385, 300]),
        `where it's scrolled to, the size of the content and of its visible part: ${JSON.stringify(sent)}`,
    );
    assert(!event.defaultPrevented && !event.propagationStopped, "and the browser goes on as usual");
}

function testScrollListener() {
    console.log("\n--- Test scroll 2: the listener ---");
    const node = new MockNode();
    const nodes = { get: () => node, hasInitNodes: () => false } as unknown as MapNodes;
    const manager = new CallbackManager(() => ({}) as any);

    manager.add(nodes, 1, 'scroll', 30);
    manager.remove(nodes, 1, 'scroll', 30);

    assert(JSON.stringify(node.added) === JSON.stringify([['scroll', false]]), "goes on the scroll of the element");
    assert(JSON.stringify(node.removed) === JSON.stringify(['scroll']), "and comes off it");
}

/// An element of the page, `scrollTop` px down and `scrollLeft` px from the start of its lines.
const scrolledElement = (scrollTop: number, scrollLeft: number) => Object.assign(new MockElement(), {
    scrollTop,
    scrollLeft,
    scrollWidth: 400,
    scrollHeight: 900,
    clientWidth: 385,
    clientHeight: 300,
    addEventListener() {},
});

async function testScrollFromTheServer() {
    console.log("\n--- Test scroll 3: scrolled before the app was there ---");
    const sent: unknown[] = [];
    const wasm = {
        wasmCommand: (command: any) => sent.push([command.CallbackCall.callback_id, command.CallbackCall.value]),
    };
    const manager = new CallbackManager(() => wasm as any);
    const batch = (node: unknown, first: boolean) => ({ get: () => node, hasInitNodes: () => first }) as unknown as MapNodes;

    manager.add(batch(scrolledElement(120.5, 0), true), 1, 'scroll', 31);
    manager.add(batch(scrolledElement(0, -250), true), 2, 'scroll', 32);
    manager.add(batch(scrolledElement(0, 0), true), 3, 'scroll', 33);
    manager.add(batch(scrolledElement(50, 0), false), 4, 'scroll', 34);
    assert(sent.length === 0, "not in the middle of the batch");
    await Promise.resolve();
    assert(
        JSON.stringify(sent) === JSON.stringify([
            [31, [120.5, 0, 400, 900, 385, 300]],
            [32, [0, -250, 400, 900, 385, 300]],
        ]),
        `but right after the first one, where the scrolled ones are - and none of a later batch: ${JSON.stringify(sent)}`,
    );
}

/// The browser's `ResizeObserver`, which tells of the elements it observes when told to.
class MockResizeObserver {
    static all: MockResizeObserver[] = [];
    observed: Array<[unknown, unknown]> = [];
    disconnected = false;
    constructor(private readonly callback: () => void) {
        MockResizeObserver.all.push(this);
    }
    observe(target: unknown, options: unknown) { this.observed.push([target, options]); }
    disconnect() { this.disconnected = true; }
    tell() { this.callback(); }
}

function testResize() {
    console.log("\n--- Test resize: on_resize ---");
    (globalThis as any).ResizeObserver = MockResizeObserver;
    const node = new MockElement();
    const sent: unknown[] = [];
    const wasm = {
        wasmCommand: (command: any) => sent.push([command.CallbackCall.callback_id, command.CallbackCall.value]),
    };
    const manager = new CallbackManager(() => wasm as any);
    const nodes = { getNode: () => node } as unknown as MapNodes;

    manager.add(nodes, 1, 'resize', 40);
    const observer = MockResizeObserver.all[0];
    assert(observer !== undefined, "an observer watches the element");
    assert(
        JSON.stringify(observer?.observed) === JSON.stringify([[{}, { box: 'border-box' }]]),
        "with its border, as drawn",
    );
    observer?.tell();
    assert(
        JSON.stringify(sent) === JSON.stringify([[40, [300, 150]]]),
        `wasm gets the size of the box as drawn, as a pointer event does: ${JSON.stringify(sent)}`,
    );

    manager.remove(nodes, 1, 'resize', 40);
    assert(observer?.disconnected === true, "and it stops watching with the element gone");

    delete (globalThis as any).ResizeObserver;
}

function testChangeChecked() {
    console.log("\n--- Test change_checked: on_change_checked ---");
    // The browser's class - here it only has to be there for `instanceof`
    class MockInput {
        checked = true;
        value = 'on';
    }
    (globalThis as any).HTMLInputElement = MockInput;

    const node = new MockNode();
    const nodes = { get: () => node } as unknown as MapNodes;
    const sent: unknown[] = [];
    const wasm = { wasmCommand: (command: any) => sent.push(command.CallbackCall.value) };
    const manager = new CallbackManager(() => wasm as any);

    manager.add(nodes, 1, 'change_checked', 30);
    assert(
        JSON.stringify(node.added) === JSON.stringify([['change', false]]),
        "listens to the DOM change event",
    );

    const input = new MockInput();
    node.listeners[0]!({ target: input } as unknown as Event);
    input.checked = false;
    node.listeners[0]!({ target: input } as unknown as Event);
    assert(
        JSON.stringify(sent) === JSON.stringify([true, false]),
        "and tells wasm whether the box is checked, not its value",
    );

    manager.remove(nodes, 1, 'change_checked', 30);
    assert(JSON.stringify(node.removed) === JSON.stringify(['change']), "which it stops listening to");

    delete (globalThis as any).HTMLInputElement;
}

async function testChangeFile() {
    console.log("\n--- Test change_file: on_change_file ---");
    // The browser's class - here it only has to be there for `instanceof`
    class MockInput {
        value = 'C:\\fakepath\\tab.pdf';
        files = [{ name: 'tab.pdf', arrayBuffer: () => Promise.resolve(new Uint8Array([37, 80, 68, 70]).buffer) }];
    }
    (globalThis as any).HTMLInputElement = MockInput;

    const node = new MockNode();
    const nodes = { get: () => node } as unknown as MapNodes;
    const sent: unknown[] = [];
    const wasm = { wasmCommand: (command: any) => sent.push(command.CallbackCall.value) };
    const manager = new CallbackManager(() => wasm as any);

    manager.add(nodes, 1, 'change_file', 35);
    assert(
        JSON.stringify(node.added) === JSON.stringify([['change', false]]),
        "listens to the DOM change event",
    );

    const input = new MockInput();
    node.listeners[0]!({ target: input } as unknown as Event);
    // The files are read before they go
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert(
        JSON.stringify(sent) === JSON.stringify([[[['tab.pdf', [37, 80, 68, 70]]]]]),
        `and sends wasm each file's name and bytes: ${JSON.stringify(sent)}`,
    );
    assert(input.value === 'C:\\fakepath\\tab.pdf', "leaving the input showing the file picked into it");

    delete (globalThis as any).HTMLInputElement;
}

/// The browser's classes - here they only have to be there for `instanceof`. Node has
/// a `MessageEvent` of its own, but it takes only a `MessagePort` for the source.
class MockFrame {
    contentWindow: object;
    constructor(contentWindow: object) {
        this.contentWindow = contentWindow;
    }
}

class MockMessageEvent {
    source: object | null;
    origin = 'https://example.com';
    data: unknown;
    constructor(source: object | null, data: unknown) {
        this.source = source;
        this.data = data;
    }
}

const nodeMessageEvent = globalThis.MessageEvent;

/// The window of the page, which keeps its listeners, and the classes of the browser.
function mockBrowser(): MockNode {
    const page = new MockNode();
    (globalThis as any).window = page;
    (globalThis as any).HTMLIFrameElement = MockFrame;
    (globalThis as any).MessageEvent = MockMessageEvent;
    return page;
}

function restoreNode() {
    delete (globalThis as any).window;
    delete (globalThis as any).HTMLIFrameElement;
    (globalThis as any).MessageEvent = nodeMessageEvent;
}

function testMessage() {
    console.log("\n--- Test message: on_message ---");
    // The window of the page and the one of the page in the frame
    const page = mockBrowser();
    const inFrame = {};

    const frame = new MockFrame(inFrame);
    const nodes = { get: () => frame } as unknown as MapNodes;
    const sent: unknown[] = [];
    const wasm = { wasmCommand: (command: any) => sent.push(command.CallbackCall.value) };
    const manager = new CallbackManager(() => wasm as any);

    manager.add(nodes, 1, 'message', 40);
    assert(
        JSON.stringify(page.added) === JSON.stringify([['message', false]]),
        "listens to the messages to the window",
    );

    const post = (source: object | null, data: unknown) =>
        page.listeners[0]!(new MockMessageEvent(source, data) as unknown as Event);
    post(inFrame, { height: 338, when: new Date(0) });
    post({}, 'from another frame');
    post(null, 'from a worker');
    post(inFrame, 10n);
    post(inFrame, { tile: new Uint8Array([0, 20, 255]) });
    assert(
        JSON.stringify(sent) === JSON.stringify([
            ['https://example.com', { height: 338, when: '1970-01-01T00:00:00.000Z' }],
            ['https://example.com', null],
            ['https://example.com', { tile: [0, 20, 255] }],
        ]),
        `only from the page in the frame, as JSON can hold it: ${JSON.stringify(sent)}`,
    );

    manager.remove(nodes, 1, 'message', 40);
    assert(JSON.stringify(page.removed) === JSON.stringify(['message']), "which it stops listening to");

    let threw = false;
    try {
        manager.add({ get: () => new MockNode() } as unknown as MapNodes, 2, 'message', 41);
    } catch {
        threw = true;
    }
    assert(threw && page.added.length === 1, "and on anything but a frame it throws, listening to nothing");

    restoreNode();
}

async function testEarlyMessages() {
    console.log("\n--- Test message 2: before the app was there ---");
    const page = mockBrowser();
    const inFrame = {};

    // A frame of a page from the server, and another one, spoke before the wasm started
    keepEarlyMessage(new MockMessageEvent(inFrame, 'early') as unknown as MessageEvent);
    keepEarlyMessage(new MockMessageEvent({}, 'another frame') as unknown as MessageEvent);

    const frame = new MockFrame(inFrame);
    const nodes = { get: () => frame } as unknown as MapNodes;
    const sent: unknown[] = [];
    const wasm = { wasmCommand: (command: any) => sent.push(command.CallbackCall.value) };
    const manager = new CallbackManager(() => wasm as any);

    manager.add(nodes, 1, 'message', 41);
    assert(sent.length === 0, "not in the middle of the batch");
    manager.forgetEarlyMessages();
    await Promise.resolve();
    assert(
        JSON.stringify(sent) === JSON.stringify([['https://example.com', 'early']]),
        `but right after it, what its frame said: ${JSON.stringify(sent)}`,
    );
    assert(JSON.stringify(page.removed) === JSON.stringify(['message']), "and then no more are kept");

    keepEarlyMessage(new MockMessageEvent(inFrame, 'late') as unknown as MessageEvent);
    manager.add(nodes, 2, 'message', 42);
    await Promise.resolve();
    assert(sent.length === 1, "so a frame added later gets only what comes after");

    restoreNode();
}

/// A video 12.5 s in, with its length unknown yet, and two pieces of it in the browser: from 0
/// to 20 s and from 30 to 40 s. The browser's class - here it only has to be there for
/// `instanceof`.
class MockMedia extends MockNode {
    currentTime = 12.5;
    duration = NaN;
    paused = true;
    ended = false;
    muted = false;
    volume = 0.8;
    error: object | null = null;
    readyState = 0;
    buffered = {
        length: 2,
        start: (index: number) => [0, 30][index]!,
        end: (index: number) => [20, 40][index]!,
    };
}

function testMedia() {
    console.log("\n--- Test media: on_media ---");
    (globalThis as any).HTMLMediaElement = MockMedia;

    const element = new MockMedia();
    const nodes = { get: () => element } as unknown as MapNodes;
    const sent: unknown[] = [];
    const wasm = { wasmCommand: (command: any) => sent.push(command.CallbackCall.value) };
    const manager = new CallbackManager(() => wasm as any);

    manager.add(nodes, 1, 'media', 50);
    assert(
        JSON.stringify(element.added.map(([name]) => name)) === JSON.stringify([
            'loadedmetadata', 'durationchange', 'timeupdate', 'progress', 'play', 'playing', 'pause',
            'waiting', 'seeked', 'ended', 'volumechange', 'error',
        ]),
        "listens to the events of the media",
    );

    const fire = (type: string) => element.listeners[0]!({ type } as Event);
    fire('loadedmetadata');
    element.duration = 60;
    element.paused = false;
    fire('play');
    element.duration = Infinity;
    element.currentTime = 35;
    fire('timeupdate');
    assert(
        JSON.stringify(sent) === JSON.stringify([
            ['loadedmetadata', 12.5, null, true, false, false, 0.8, 20],
            ['play', 12.5, 60, false, false, false, 0.8, 20],
            ['timeupdate', 35, null, false, false, false, 0.8, 40],
        ]),
        `with the state after, the length null while unknown and for a stream: ${JSON.stringify(sent)}`,
    );

    manager.remove(nodes, 1, 'media', 50);
    assert(element.removed.length === 12, "which it stops listening to");

    let threw = false;
    try {
        manager.add({ get: () => new MockNode() } as unknown as MapNodes, 2, 'media', 51);
    } catch {
        threw = true;
    }
    assert(threw, "and on anything but a video or an audio it throws");

    delete (globalThis as any).HTMLMediaElement;
}

async function testMediaFromTheServer() {
    console.log("\n--- Test media 2: loaded before the app was there ---");
    (globalThis as any).HTMLMediaElement = MockMedia;

    const loaded = new MockMedia();
    loaded.readyState = 1;
    const broken = new MockMedia();
    broken.error = { code: 4 };
    const sent: unknown[] = [];
    const wasm = { wasmCommand: (command: any) => sent.push(command.CallbackCall.value) };
    const manager = new CallbackManager(() => wasm as any);

    manager.add({ get: () => loaded } as unknown as MapNodes, 1, 'media', 52);
    manager.add({ get: () => broken } as unknown as MapNodes, 2, 'media', 53);
    manager.add({ get: () => new MockMedia() } as unknown as MapNodes, 3, 'media', 54);
    assert(sent.length === 0, "not in the middle of the batch");
    await Promise.resolve();
    assert(
        JSON.stringify(sent.map((value) => (value as unknown[])[0])) === JSON.stringify(['loadedmetadata', 'error']),
        `but right after it, what the media said by then: ${JSON.stringify(sent)}`,
    );

    delete (globalThis as any).HTMLMediaElement;
}

testClickPayload();
testDefaultActionIsLeftToTheBrowser();
testPreventDefault();
testStopPropagation();
testReplyWithoutFlags();
testWheelPayload();
testWheelScrollsByDefault();
testWheelPreventDefault();
testWheelStopPropagation();
testWheelListenerIsNotPassive();
testPointerPayload();
testPointerByDefault();
testPointerCapture();
testPointerCaptureOfAPointerThatIsUp();
testPointerRelease();
testPointerReleaseOfAPointerNotCaptured();
testPointerFlags();
testPointerListeners();
testScrollPayload();
testScrollListener();
void testScrollFromTheServer();
testResize();
testChangeChecked();
void testChangeFile();
testMessage();
void testEarlyMessages();
testMedia();
void testMediaFromTheServer();
