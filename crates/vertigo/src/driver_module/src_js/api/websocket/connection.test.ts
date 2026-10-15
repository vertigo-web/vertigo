// The socket loop: connecting, reconnecting, and letting go of the socket on dispose.

import { SocketConnection } from "./connection";

const logError = console.error;

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        logError(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

// --- MOCKS ---

/// Stands in for the browser's WebSocket. The test decides when a socket opens or the server
/// closes it. `close()` ends in a `close` event, after an `error` one if the socket was still
/// connecting - as in a browser.
class FakeWebSocket {
    static instances: FakeWebSocket[] = [];
    opened = false;
    closed = false;
    private listeners: Map<string, Array<(event: unknown) => void>> = new Map();

    constructor(readonly url: string) {
        FakeWebSocket.instances.push(this);
    }

    addEventListener(type: string, listener: (event: unknown) => void) {
        this.listeners.set(type, [...(this.listeners.get(type) ?? []), listener]);
    }

    send(_message: string) {}

    close() {
        if (this.closed) {
            return;
        }
        this.closed = true;
        setTimeout(() => {
            if (!this.opened) {
                this.emit('error');
            }
            this.emit('close');
        }, 0);
    }

    open() {
        this.opened = true;
        this.emit('open');
    }

    emit(type: string, event: unknown = {}) {
        for (const listener of this.listeners.get(type) ?? []) {
            listener(event);
        }
    }
}

(globalThis as any).WebSocket = FakeWebSocket;
// The loop narrates every step; the PASS lines are what matters here.
console.info = () => {};
// What the loop reports as errors.
const errors: Array<unknown> = [];
console.error = (...args: Array<unknown>) => {
    errors.push(args);
};

const RETRY_MS = 10;
const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/// Starts a socket loop against fresh mocks, with what it reports to the app.
const start = () => {
    FakeWebSocket.instances = [];
    errors.length = 0;
    const events: Array<string> = [];
    const controller = SocketConnection.startSocket('ws://test', 1000, RETRY_MS, (message) => {
        events.push(message.type);
    });
    const latest = (): FakeWebSocket => {
        const socket = FakeWebSocket.instances[FakeWebSocket.instances.length - 1];
        if (socket === undefined) {
            throw new Error('no socket was created');
        }
        return socket;
    };
    return { controller, events, latest };
};

// --- TESTS ---

async function testDisposeWhileConnecting() {
    console.log("\n--- Test socket 1: dispose before the socket opens ---");
    const { controller, events, latest } = start();
    const socket = latest();

    controller.dispose();
    assert(socket.closed, "dispose closes a socket that is still connecting");
    await sleep(0);
    assert(errors.length === 0, "and reports no error for it");

    // The worst case: it opens all the same.
    socket.open();
    await sleep(RETRY_MS * 5);
    assert(!events.includes('socket'), "the app never hears of it");
    assert(FakeWebSocket.instances.length === 1, "and no new attempt starts");
}

async function testDisposeWhileOpen() {
    console.log("\n--- Test socket 2: dispose an open socket ---");
    const { controller, events, latest } = start();
    const socket = latest();
    socket.open();
    await sleep(0);
    assert(events.join() === 'socket', "the app gets the open socket");

    controller.dispose();
    await sleep(RETRY_MS * 5);
    assert(socket.closed, "dispose closes it");
    assert(events.join() === 'socket,close', "the app hears it closed");
    assert(FakeWebSocket.instances.length === 1, "and no new attempt starts");
}

async function testReconnectAfterServerClose() {
    console.log("\n--- Test socket 3: the server closes the socket ---");
    const { controller, events, latest } = start();
    latest().open();
    await sleep(0);

    latest().emit('close');
    await sleep(RETRY_MS * 5);
    assert(FakeWebSocket.instances.length === 2, "a new attempt starts after the delay");
    assert(events.join() === 'socket,close', "after telling the app the socket closed");

    controller.dispose();
    assert(latest().closed, "dispose closes the new attempt too");
}

async function testDisposeDuringReconnectDelay() {
    console.log("\n--- Test socket 4: dispose while waiting to reconnect ---");
    const { controller, latest } = start();
    latest().open();
    await sleep(0);
    latest().emit('close');
    await sleep(0);

    controller.dispose();
    await sleep(RETRY_MS * 5);
    assert(FakeWebSocket.instances.length === 1, "no new attempt starts");
}

async function main() {
    await testDisposeWhileConnecting();
    await testDisposeWhileOpen();
    await testReconnectAfterServerClose();
    await testDisposeDuringReconnectDelay();
}

main();
