import { EventEmitter } from "./event_emiter";
import { PromiseBoxRace } from "./promise";

const timeout = async (timeout: number): Promise<void> => {
    return new Promise((resolve: (data: void) => void) => {
        setTimeout(resolve, timeout);
    });
};


const reconnectDelay = async (label: string, timeout_retry: number): Promise<void> => {
    console.info(`${label} wait ${timeout_retry}ms`);
    await timeout(timeout_retry);
    console.info(`${label} go forth`);
};

interface CloseInfo {
    code: number,
    reason: string,
}

export type SocketEventType = {
    type: 'message',
    message: string,
} | {
    type: 'socket',
    socket: SocketConnection
} | {
    // A connection closed (`connected`) or an attempt to connect failed.
    type: 'close',
    code: number,
    reason: string,
    connected: boolean,
};

// For a close, the answer is how many milliseconds to wait before connecting again - anything
// but a number stops the loop.
export type OnMessageType = (message: SocketEventType) => unknown;
export type UnsubscribeFnType = () => void;

interface OpenSocketResult {
    // Closes the socket whether it is still connecting or already open.
    close: () => void,
    socket: Promise<SocketConnection | null>,
    done: Promise<CloseInfo>,
}

// Messages go through the `SocketConnection` the loop reports, so the loop's own controller
// only stops it.
export interface SocketConnectionController {
    dispose: UnsubscribeFnType
}

class LogContext {
    public constructor(private host: string) {}
    public formatLog = (message: string): string => `Socket ${this.host} ==> ${message}`;
}
export class SocketConnection {
    private readonly eventMessage: EventEmitter<string>;
    public readonly close: () => void;
    public readonly send: (message: string) => void;

    private constructor(
        close: () => void,
        send: (message: string) => void,
    ) {
        this.eventMessage = new EventEmitter();
        this.close = close;
        this.send = send;
    }

    private static connect(
        log: LogContext,
        host: string,
        timeout: number,
    ): OpenSocketResult {
        const result = new PromiseBoxRace<SocketConnection | null>();
        const done = new PromiseBoxRace<CloseInfo>();
        const socket = new WebSocket(host);
        let isClose: boolean = false;

        console.info(log.formatLog('starting ...'));

        // Called with the browser's close event, or without one when we close the socket
        // ourselves. Of our own closes the app only hears of the one when the page goes into
        // the back/forward cache - as 1001, going away.
        const closeSocket = (event?: CloseEvent): void => {
            if (isClose) {
                return;
            }

            console.info(log.formatLog('close'));

            isClose = true;
            result.resolve(null);
            done.resolve({ code: event?.code ?? 1001, reason: event?.reason ?? '' });
            socket.close();
        };


        const socketConnection = new SocketConnection(
            closeSocket,
            (message: string) => {
                if (isClose) {
                    return;
                }
                socket.send(message);
            }
        );

        setTimeout(() => {
            if (result.isFulfilled() === false) {
                console.error(log.formatLog(`timeout (${timeout}ms)`));
                closeSocket();
            }
        }, timeout);

        const onOpen = (): void => {
            console.info(log.formatLog('open'));
            result.resolve(socketConnection);
        };

        const onError = (error: Event): void => {
            if (isClose) {
                // Comes from our own close, f. ex. of a socket that was still connecting.
                return;
            }

            // The close event comes next, with the code.
            console.error(log.formatLog('error'), error);
        };

        const onMessage = (event: MessageEvent): void => {
            if (isClose) {
                return;
            }

            const dataRaw = event.data;

            if (typeof dataRaw === 'string') {
                socketConnection.eventMessage.trigger(dataRaw);
                return;
            }

            console.error(log.formatLog('onMessage - expected string'), dataRaw);
        };

        socket.addEventListener('open', onOpen);
        socket.addEventListener('error', onError);
        socket.addEventListener('close', closeSocket);
        socket.addEventListener('message', onMessage);

        return {
            close: closeSocket,
            socket: result.promise,
            done: done.promise
        };
    }

    public static startSocket(
        host: string,
        timeout_connection: number,
        onMessage: OnMessageType,
    ): SocketConnectionController {
        let isConnect: boolean = true;
        // The latest attempt, still connecting or open. `dispose` has to close it in either
        // state - a socket that finished connecting after `dispose` would otherwise stay open,
        // unseen by the app, until the page goes away.
        let closeCurrent: () => void = () => {};

        const log = new LogContext(host);

        (async (): Promise<void> => {
            while (isConnect) {
                const openSocketResult = SocketConnection.connect(log, host, timeout_connection);
                closeCurrent = openSocketResult.close;

                const socket = await openSocketResult.socket;

                if (!isConnect) {
                    // Disposed while connecting, and `dispose` has closed the socket.
                    console.info(log.formatLog('disconnect (3)'));
                    return;
                }

                let delay: unknown;

                if (socket === null) {
                    // Browsers tell no more about a failed attempt than 1006.
                    delay = onMessage({
                        type: 'close',
                        code: 1006,
                        reason: '',
                        connected: false,
                    });
                } else {
                    onMessage({
                        type: 'socket',
                        socket
                    });

                    socket.eventMessage.on(message => {
                        onMessage({
                            type: 'message',
                            message
                        });
                    });

                    const closed = await openSocketResult.done;

                    delay = onMessage({
                        type: 'close',
                        ...closed,
                        connected: true,
                    });

                    if (!isConnect) {
                        console.info(log.formatLog('disconnect (1)'));
                        return;
                    }
                }

                if (typeof delay !== 'number') {
                    console.info(log.formatLog('stop'));
                    return;
                }

                await reconnectDelay(log.formatLog('reconnect'), delay);
            }

            console.info(log.formatLog('disconnect (2)'));
        })().catch((error) => {
            console.error(error);
        });

        return {
            dispose: (): void => {
                isConnect = false;
                closeCurrent();
            }
        };
    }
}
