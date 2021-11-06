const assertNeverMessage = (data: never): never => {
    console.error(data);
    throw Error('unknown message');
};

const timeout = async (timeout: number): Promise<void> => {
    return new Promise((resolve: (data: void) => void) => {
        setTimeout(resolve, timeout);
    });
};


export class EventEmmiter<T> {
    private events: Set<(param: T) => void>;

    constructor() {
        this.events = new Set()
    }

    on(callback: (param: T) => void) {
        let isActive = true;

        const onExec = (param: T) => {
            if (isActive) {
                callback(param);
            }
        };

        this.events.add(onExec);

        return () => {
            isActive = false;
            this.events.delete(onExec);
        };
    }

    trigger(param: T) {
        const eventsCopy = Array.from(this.events.values())

        for (const itemCallbackToRun of eventsCopy) {
            try {
                itemCallbackToRun(param);
            } catch (err) {
                console.error(err);
            }
        }
    }

    get size(): number {
        return this.events.size;
    }
}

type ResolveFn<T> = (data: T) => void;
type RejectFn = (err: unknown) => void;

interface PromiseResolveReject<T> {
    readonly resolve: (value: T) => void,
    readonly reject: (err: unknown) => void,
};

const createPromiseValue = <T>(): [PromiseResolveReject<T>, Promise<T>] => {
    let resolve: ResolveFn<T> | null = null;
    let reject: RejectFn | null = null;

    const promise: Promise<T> = new Promise((localResolve: ResolveFn<T>, localReject: RejectFn) => {
        resolve = localResolve;
        reject = localReject;
    });

    if (resolve === null) {
        throw Error('createPromiseValue - resolve is null');
    }

    if (reject === null) {
        throw Error('createPromiseValue - reject is null');
    }

    const promiseValue = {
        resolve,
        reject,
    };

    return [promiseValue, promise];
};

class PromiseBoxRace<T> {
    private promiseResolveReject: PromiseResolveReject<T> | null = null;
    readonly promise: Promise<T>;

    constructor() {
        const [promiseResolveReject, promise] = createPromiseValue<T>();

        this.promiseResolveReject = promiseResolveReject;
        this.promise = promise;
    }

    resolve = (value: T) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;

        if (promiseResolveReject === null) {
            return;
        }

        promiseResolveReject.resolve(value);
    }

    reject = (err?: unknown) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;

        if (promiseResolveReject === null) {
            return;
        }

        promiseResolveReject.reject(err);
    }

    isFulfilled = (): boolean => {
        return this.promiseResolveReject === null;
    }
}

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

export class DriverWebsocketJs {
    private readonly controllerList: Map<BigInt, SocketConnectionController>;
    private readonly socket: Map<BigInt, SocketConnection>;
    private readonly callback_socket: (connection_id: BigInt) => void;
    private readonly callback_message: (connection_id: BigInt, message: String) => void;
    private readonly callback_close: (connection_id: BigInt) => void;

    constructor(
        callback_socket: (connection_id: BigInt) => void,
        callback_message: (connection_id: BigInt, message: String) => void,
        callback_close: (connection_id: BigInt) => void,
    ) {
        this.controllerList = new Map();
        this.socket = new Map();
        this.callback_socket = callback_socket;
        this.callback_message = callback_message;
        this.callback_close = callback_close;
    }

    public register_callback(
        host: string,
        callback_id: BigInt,
    ) {
        let controller = SocketConnection.startSocket(
            host,
            5000,                   //timeout connection 
            3000,                   //timeout reconnection
            (message) => {

                if (this.controllerList.has(callback_id) === false) {
                    return;
                }
                    
                if (message.type === 'socket') {
                    this.socket.set(callback_id, message.socket);
                    this.callback_socket(callback_id);
                    return;
                }
        
                if (message.type === 'message') {
                    this.callback_message(callback_id, message.message);
                    return;
                }

                if (message.type === 'close') {
                    this.callback_close(callback_id);
                    this.socket.delete(callback_id);
                    return;
                }

                return assertNeverMessage(message);
            }
        );

        this.controllerList.set(callback_id, controller);
    }

    public unregister_callback(callback_id: BigInt) {
        const controller = this.controllerList.get(callback_id);

        if (controller === undefined) {
            console.error('Expected controller');
            return;
        }

        controller.dispose();
        this.controllerList.delete(callback_id);
    }

    public send_message(
        callback_id: BigInt,
        message: string,
    ) {
        const socket = this.socket.get(callback_id);

        if (socket === undefined) {
            console.error(`Missing socket connection for callback_id=${callback_id}`);
        } else {
            socket.send(message);
        }
    }
}
