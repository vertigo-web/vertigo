// The click handler: what a `ClickEvent` reply does to the browser's own handling of the click.

import { click } from "./callbackManager";

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

testDefaultActionIsLeftToTheBrowser();
testPreventDefault();
testStopPropagation();
testReplyWithoutFlags();
