// The click, wheel and pointer handlers: what a `ClickEvent`, `WheelEvent` or `PointerEvent`
// reply does to the browser's own handling of the event, and what the handlers tell wasm.

import { CallbackManager, click, pointer, wheel } from "./callbackManager";
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

/// A click whose `on_click` in wasm replied with `reply` - what `ClickEvent` is serialized to.
function clickReplying(reply: unknown): MockEvent {
    const event = new MockEvent();
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
    addEventListener(name: string, _listener: unknown, options: unknown) {
        this.added.push([name, options]);
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
