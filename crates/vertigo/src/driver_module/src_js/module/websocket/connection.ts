import { EventEmmiter } from "./event_emiter";
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

export type SocketEventType = {
    type: 'message',
    message: string,
} | {
    type: 'socket',
    socket: SocketConnection
} | {
    type: 'close',
};

export type OnMessageType = (message: SocketEventType) => void;
export type UnsubscribeFnType = () => void;

interface OpenSocketResult {
    socket: Promise<SocketConnection | null>,
    done: Promise<void>,
}

export interface SocketConnectionController {
    send: (message: string) => void,
    dispose: UnsubscribeFnType
}

class LogContext {
    public constructor(private host: string) {}
    public formatLog = (message: string): string => `Socket ${this.host} ==> ${message}`;
}
export class SocketConnection {
    private readonly eventMessage: EventEmmiter<string>;
    public readonly close: () => void;
    public readonly send: (message: string) => void;

    private constructor(
        close: () => void,
        send: (message: string) => void,
    ) {
        this.eventMessage = new EventEmmiter();
        this.close = close;
        this.send = send;
    }

    private static connect(
        log: LogContext,
        host: string,
        timeout: number,
    ): OpenSocketResult {
        const result = new PromiseBoxRace<SocketConnection | null>();
        const done = new PromiseBoxRace<void>();
        const socket = new WebSocket(host);
        let isClose: boolean = false;

        console.info(log.formatLog('starting ...'));

        const closeSocket = (): void => {
            if (isClose) {
                return;
            }

            console.info(log.formatLog('close'));

            isClose = true;
            result.resolve(null);
            done.resolve();
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
            console.error(log.formatLog('error'), error);
            closeSocket();
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
            socket: result.promise,
            done: done.promise
        };
    }

    public static startSocket(
        host: string,
        timeout_connection: number,
        timeout_retry: number,
        onMessage: OnMessageType,
    ): SocketConnectionController {
        let isConnect: boolean = true;
        let socketConnection: SocketConnection | null = null;

        const log = new LogContext(host);

        (async (): Promise<void> => {
            while (isConnect) {
                const openSocketResult = SocketConnection.connect(log, host, timeout_connection);

                const socket = await openSocketResult.socket;

                if (socket === null) {
                    await reconnectDelay(log.formatLog('reconnect after error'), timeout_retry);
                    continue;
                }

                socketConnection = socket;
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

                await openSocketResult.done;

                onMessage({
                    type: 'close'
                });

                if (!isConnect) {
                    console.info(log.formatLog('disconnect (1)'));
                    return;
                }

                await reconnectDelay(log.formatLog('reconnect after close'), timeout_retry);
            }

            console.info(log.formatLog('disconnect (2)'));
        })().catch((error) => {
            console.error(error);
        });

        return {
            send: (message: string): void => {
                if (socketConnection === null) {
                    console.error('send fail - missing connection', message);
                } else {
                    socketConnection.send(message);
                }
            },
            dispose: (): void => {
                isConnect = false;
                socketConnection?.close();
            }
        };
    }
}
