// The socket loop: connecting, reconnecting, and letting go of the socket on dispose.

import { SocketConnection, SocketEventType } from "./connection";
import { FakeWebSocket, sleep } from "../../testing/fake_websocket";

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

(globalThis as any).WebSocket = FakeWebSocket;
// The loop narrates every step; the PASS lines are what matters here.
console.info = () => {};
// What the loop reports as errors.
const errors: Array<unknown> = [];
console.error = (...args: Array<unknown>) => {
    errors.push(args);
};

const RETRY_MS = 10;

type Close = Extract<SocketEventType, { type: 'close' }>;

/// Starts a socket loop against fresh mocks. `events` is what the loop reports to the app, a close
/// as `close:<code>:<reason>:<connected>` and a message as `message:<text>` or `message:[<bytes>]`.
/// `reply` answers a close as the app does: with the wait before the next attempt, or anything
/// else to stop.
const start = (reply: (close: Close) => unknown = () => RETRY_MS) => {
    FakeWebSocket.instances = [];
    errors.length = 0;
    const events: Array<string> = [];
    const controller = SocketConnection.startSocket('ws://test', 1000, (message) => {
        if (message.type === 'message') {
            const data = message.message;
            events.push(`message:${typeof data === 'string' ? data : `[${Array.from(data)}]`}`);
            return;
        }
        if (message.type !== 'close') {
            events.push(message.type);
            return;
        }
        events.push(`close:${message.code}:${message.reason}:${message.connected}`);
        return reply(message);
    });
    return { controller, events, latest: FakeWebSocket.latest };
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
    assert(events.length === 0, "the app never hears of it");
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
    assert(events.join() === 'socket,close:1001::true', "the app hears it went away (1001)");
    assert(FakeWebSocket.instances.length === 1, "and no new attempt starts");
}

async function testReconnectAfterServerClose() {
    console.log("\n--- Test socket 3: the server closes the socket ---");
    const { controller, events, latest } = start();
    latest().open();
    await sleep(0);

    latest().emit('close', { code: 4000, reason: 'bye' });
    await sleep(RETRY_MS * 5);
    assert(
        events.join() === 'socket,close:4000:bye:true',
        "the app gets the server's code and reason"
    );
    assert(FakeWebSocket.instances.length === 2, "and a new attempt starts after the delay");

    controller.dispose();
    assert(latest().closed, "dispose closes the new attempt too");
}

async function testDisposeDuringReconnectDelay() {
    console.log("\n--- Test socket 4: dispose while waiting to reconnect ---");
    const { controller, latest } = start();
    latest().open();
    await sleep(0);
    latest().emit('close', { code: 1000, reason: '' });
    await sleep(0);

    controller.dispose();
    await sleep(RETRY_MS * 5);
    assert(FakeWebSocket.instances.length === 1, "no new attempt starts");
}

async function testFailedAttempt() {
    console.log("\n--- Test socket 5: an attempt to connect fails ---");
    const { controller, events, latest } = start();

    // What a browser does when nobody answers.
    latest().emit('error');
    latest().emit('close', { code: 1006, reason: '' });
    await sleep(RETRY_MS * 5);
    assert(events.join() === 'close:1006::false', "the app hears of the failed attempt");
    assert(FakeWebSocket.instances.length === 2, "and the loop tries again after its delay");

    controller.dispose();
}

async function testTheAppDecidesTheWait() {
    console.log("\n--- Test socket 6: the app's answer to a close ---");
    const { controller, latest } = start(() => 200);
    latest().open();
    await sleep(0);

    latest().emit('close', { code: 1000, reason: '' });
    await sleep(50);
    assert(FakeWebSocket.instances.length === 1, "the loop waits as long as the app said");
    await sleep(250);
    assert(FakeWebSocket.instances.length === 2, "and then tries again");
    controller.dispose();

    const stopped = start(() => null);
    stopped.latest().open();
    await sleep(0);
    stopped.latest().emit('close', { code: 4004, reason: 'no such world' });
    await sleep(RETRY_MS * 5);
    assert(FakeWebSocket.instances.length === 1, "without a wait from the app the loop stops");
}

async function testBinaryFrames() {
    console.log("\n--- Test socket 7: binary frames ---");
    const sockets: Array<SocketConnection> = [];
    FakeWebSocket.instances = [];
    const events: Array<string> = [];
    const controller = SocketConnection.startSocket('ws://test', 1000, (message) => {
        if (message.type === 'socket') {
            sockets.push(message.socket);
        }
        if (message.type === 'message') {
            const data = message.message;
            events.push(typeof data === 'string' ? data : `[${Array.from(data)}]`);
        }
        return null;
    });
    const socket = FakeWebSocket.latest();
    assert(socket.binaryType === 'arraybuffer', "binary frames come as ArrayBuffers, read at once");

    socket.open();
    await sleep(0);
    socket.emit('message', { data: new Uint8Array([1, 2, 255]).buffer });
    socket.emit('message', { data: '{"n": 1}' });
    assert(events.join() === '[1,2,255],{"n": 1}', "the app gets the bytes, and text as before");

    sockets[0]?.send(new Uint8Array([7, 8]));
    const sent = socket.sent[0];
    assert(
        sent instanceof Uint8Array && Array.from(sent).join() === '7,8',
        "bytes go out as they are, in a binary frame"
    );
    controller.dispose();
}

async function main() {
    await testDisposeWhileConnecting();
    await testDisposeWhileOpen();
    await testReconnectAfterServerClose();
    await testDisposeDuringReconnectDelay();
    await testFailedAttempt();
    await testTheAppDecidesTheWait();
    await testBinaryFrames();
}

main();
