import { JsJsonType } from "../../jsjson";
import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { SocketConnection, SocketConnectionController } from "./connection";

const wireStringToJsJson = (raw: string): JsJsonType => {
    try {
        return JSON.parse(raw) as JsJsonType;
    } catch {
        console.error('Failed to parse websocket message', raw);
        throw Error(raw);
    }
};

const jsJsonToWebSocketWire = (value: JsJsonType): string => {
    return JSON.stringify(value);
};

const assertNeverMessage = (data: never): never => {
    console.error(data);
    throw Error('unknown message');
};

type CommandType = 'Connected' | 'Disconnected' | {
    'Message': {
        message: JsJsonType,
    }
}
const wasmCallback = (wasm: ModuleControllerType<ExportType>, callbackId: CallbackId, command: CommandType) => {
    wasm.wasmCommand({
        'Websocket': {
            callback: callbackId,
            message: command,
        }
    })
};


export class DriverWebsocket {
    private getWasm: () => ModuleControllerType<ExportType>;
    private readonly controllerList: Map<CallbackId, SocketConnectionController>;
    private readonly socket: Map<CallbackId, SocketConnection>;
    // Where each registered socket connects, to connect it again when the page comes back.
    private readonly hosts: Map<CallbackId, string>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.controllerList = new Map();
        this.socket = new Map();
        this.hosts = new Map();

        // A page kept in the back/forward cache must not keep its sockets: the browser answers
        // the server's pings for it, so the server would go on sending to a page nobody sees.
        // The app gets a close now and a new connection once the page is back, as on any
        // reconnect.
        window.addEventListener('pagehide', (event) => {
            if (event.persisted) {
                for (const controller of this.controllerList.values()) {
                    controller.dispose();
                }
            }
        });

        window.addEventListener('pageshow', (event) => {
            if (event.persisted) {
                for (const [callback_id, host] of this.hosts) {
                    this.start(host, callback_id);
                }
            }
        });
    }

    public websocket_register_callback = (
        host: string,
        callback_id: CallbackId,
    ) => {
        this.hosts.set(callback_id, host);
        this.start(host, callback_id);
    }

    private start(host: string, callback_id: CallbackId) {
        const wasm = this.getWasm();

        // The loop this one replaces, if any, stops here.
        this.controllerList.get(callback_id)?.dispose();

        const controller: SocketConnectionController = SocketConnection.startSocket(
            host,
            5000,                   //timeout connection
            3000,                   //timeout reconnection
            (message) => {

                // Only the latest loop of a registered socket speaks for it.
                if (this.controllerList.get(callback_id) !== controller) {
                    return;
                }

                if (message.type === 'socket') {
                    this.socket.set(callback_id, message.socket);
                    wasmCallback(wasm, callback_id, 'Connected');
                    return;
                }

                if (message.type === 'message') {
                    wasmCallback(wasm, callback_id, {
                        'Message': {
                            message: wireStringToJsJson(message.message)
                        }
                    });
                    return;
                }

                if (message.type === 'close') {
                    this.socket.delete(callback_id);
                    wasmCallback(wasm, callback_id, 'Disconnected');
                    return;
                }

                return assertNeverMessage(message);
            }
        );

        this.controllerList.set(callback_id, controller);
    }

    public websocket_unregister_callback = (callback_id: CallbackId) => {
        const controller = this.controllerList.get(callback_id);

        if (controller === undefined) {
            console.error('Expected controller');
            return;
        }

        controller.dispose();
        this.controllerList.delete(callback_id);
        this.hosts.delete(callback_id);
    }

    public websocket_send_message = (
        callback_id: CallbackId,
        message: JsJsonType,
    ) => {
        const socket = this.socket.get(callback_id);

        if (socket === undefined) {
            console.error(`Missing socket connection for callback_id=${callback_id}`);
        } else {
            socket.send(jsJsonToWebSocketWire(message));
        }
    }
}
