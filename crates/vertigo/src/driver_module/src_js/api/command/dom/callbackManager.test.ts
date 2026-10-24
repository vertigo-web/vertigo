// The click, wheel and pointer handlers: what a `ClickEvent`, `WheelEvent` or `PointerEvent`
// reply does to the browser's own handling of the event, and what the handlers tell wasm. And
// `change_checked`, which tells wasm whether a box is checked, and `message`, which passes on
// the messages from the page in a frame.

import { CallbackManager, click, keepEarlyMessage, pointer, wheel } from "./callbackManager";
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
        return null as any;
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
    currentTarget = {
        getBoundingClientRect: () => ({ left: 100, top: 200, width: 300, height: 150 }),
    };
}

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
/// capture.
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
    currentTarget = {
        getBoundingClientRect: () => ({ left: 100, top: 200, width: 300, height: 150 }),
        setPointerCapture: (pointerId: number) => { this.captured.push(pointerId); },
    };
}

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
    const { event } = pointerReplying({ stop_propagation: false, prevent_default: false, capture_pointer: false });
    assert(!event.defaultPrevented && !event.propagationStopped, "the browser goes on as usual");
    assert(event.captured.length === 0, "and captures nothing");
}

function testPointerCapture() {
    console.log("\n--- Test pointer 3: PointerEvent::capture_pointer ---");
    const { event } = pointerReplying({ stop_propagation: false, prevent_default: false, capture_pointer: true });
    assert(JSON.stringify(event.captured) === JSON.stringify([7]), "the element with the listener captures the pointer");
    assert(!event.defaultPrevented && !event.propagationStopped, "and that's all");
}

function testPointerCaptureOfAPointerThatIsUp() {
    console.log("\n--- Test pointer 3b: capture_pointer of a pointer that's up ---");
    const event = new MockPointerEvent();
    event.currentTarget.setPointerCapture = () => { throw new Error('NotFoundError'); };
    let threw = false;
    try {
        pointer(event as unknown as Event, () => ({ stop_propagation: true, prevent_default: false, capture_pointer: true }) as any);
    } catch {
        threw = true;
    }
    assert(!threw, "the browser's error goes to the console, not out of the listener");
    assert(event.propagationStopped, "and the other flags still apply");
}

function testPointerFlags() {
    console.log("\n--- Test pointer 4: prevent_default and stop_propagation ---");
    const { event } = pointerReplying({ stop_propagation: true, prevent_default: true, capture_pointer: false });
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
    assert(
        JSON.stringify(sent) === JSON.stringify([
            ['https://example.com', { height: 338, when: '1970-01-01T00:00:00.000Z' }],
            ['https://example.com', null],
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
testPointerFlags();
testPointerListeners();
testChangeChecked();
testMessage();
testEarlyMessages();
