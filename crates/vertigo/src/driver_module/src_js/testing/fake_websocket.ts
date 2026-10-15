// Test-only: `testing/` holds what the `*.test.ts` files share. The driver never imports it, so
// none of it ends up in `wasm_run.js`.
//
// A stand-in for the browser's WebSocket, shared by the websocket tests.

/// The test decides when a socket opens or the server closes it. `close()` ends in a `close`
/// event, after an `error` one if the socket was still connecting - as in a browser.
export class FakeWebSocket {
    static instances: FakeWebSocket[] = [];
    opened = false;
    closed = false;
    private listeners: Map<string, Array<(event: unknown) => void>> = new Map();

    constructor(readonly url: string) {
        FakeWebSocket.instances.push(this);
    }

    /// The socket created last.
    static latest(): FakeWebSocket {
        const socket = FakeWebSocket.instances[FakeWebSocket.instances.length - 1];
        if (socket === undefined) {
            throw new Error('no socket was created');
        }
        return socket;
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

export const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
