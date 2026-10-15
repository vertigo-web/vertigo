// The websocket driver: what the app hears from its sockets, and the sockets of a page that
// goes into the back/forward cache and comes back.

import { DriverWebsocket } from "./websocket";
import { FakeWebSocket, sleep } from "../../testing/fake_websocket";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

// --- MOCKS ---

(globalThis as any).WebSocket = FakeWebSocket;
(globalThis as any).window = new EventTarget();
// The socket loops narrate every step; the PASS lines are what matters here.
console.info = () => {};

/// What the driver tells the app, as `callback:message`.
const commands: Array<string> = [];
const wasm = {
    wasmCommand: (command: any) => {
        const { callback, message } = command.Websocket;
        commands.push(`${callback}:${typeof message === 'string' ? message : Object.keys(message)[0]}`);
        return null;
    },
};

const driver = new DriverWebsocket(() => wasm as any);

/// The page going into the back/forward cache (`pagehide`) or coming back from it (`pageshow`),
/// or with `persisted` false, being unloaded or loaded.
const pageTransition = (type: 'pagehide' | 'pageshow', persisted: boolean) => {
    const event = new Event(type);
    Object.defineProperty(event, 'persisted', { value: persisted });
    window.dispatchEvent(event);
};

// --- TESTS ---

async function testCachedPageClosesItsSockets() {
    console.log("\n--- Test driver 1: the page goes into the back/forward cache ---");
    driver.websocket_register_callback('ws://test/a', 1);
    const socket = FakeWebSocket.latest();
    socket.open();
    await sleep(0);
    assert(commands.join() === '1:Connected', "the app gets the connection");

    pageTransition('pagehide', true);
    assert(socket.closed, "the socket closes when the page goes into the cache");
    await sleep(0);
    assert(commands.join() === '1:Connected,1:Disconnected', "and the app hears it closed");
}

async function testPageBackFromCacheConnectsAgain() {
    console.log("\n--- Test driver 2: the page comes back ---");
    const before = FakeWebSocket.instances.length;
    pageTransition('pageshow', true);
    assert(FakeWebSocket.instances.length === before + 1, "the socket connects again at once");
    const socket = FakeWebSocket.latest();
    assert(socket.url === 'ws://test/a', "to the same address");

    // Separate tasks, as in a browser.
    socket.open();
    await sleep(0);
    socket.emit('message', { data: '{"n": 1}' });
    await sleep(0);
    assert(
        commands.slice(-2).join() === '1:Connected,1:Message',
        "the app gets the new connection and its messages"
    );
}

async function testUnloadLeavesSocketsToTheBrowser() {
    console.log("\n--- Test driver 3: the page is unloaded, not cached ---");
    const socket = FakeWebSocket.latest();
    pageTransition('pagehide', false);
    assert(!socket.closed, "the driver leaves the sockets to the browser");
}

async function testDroppedSocketStaysClosed() {
    console.log("\n--- Test driver 4: a socket the app dropped before the page went away ---");
    driver.websocket_register_callback('ws://test/b', 2);
    FakeWebSocket.latest().open();
    await sleep(0);
    driver.websocket_unregister_callback(2);

    pageTransition('pagehide', true);
    const before = FakeWebSocket.instances.length;
    pageTransition('pageshow', true);
    const addresses = FakeWebSocket.instances.slice(before).map((socket) => socket.url);
    assert(addresses.join() === 'ws://test/a', "only the sockets still registered connect again");
}

async function main() {
    await testCachedPageClosesItsSockets();
    await testPageBackFromCacheConnectsAgain();
    await testUnloadLeavesSocketsToTheBrowser();
    await testDroppedSocketStaysClosed();
}

// Every attempt arms the driver's 5 s connection timeout, which would keep node waiting for
// nothing once the tests are done.
main().then(() => (globalThis as any).process.exit(0));
