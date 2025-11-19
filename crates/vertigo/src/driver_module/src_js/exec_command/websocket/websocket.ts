import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { SocketConnection, SocketConnectionController } from "./connection";


const assertNeverMessage = (data: never): never => {
    console.error(data);
    throw Error('unknown message');
};

/*
pub enum WebsocketMessageFromBrowser {
    Connected,
    Message {
        message: String,
    },
    Disconnected,
}


pub enum CommandForWasm {
    Websocket {
        callback: CallbackId,
        message: WebsocketMessageFromBrowser,
    }
}
*/

type CommandType = 'Connected' | 'Disconnected' | {
    'Message': {
        message: string,
    }
}
const wasmCallback = (wasm: ModuleControllerType<ExportType>, callbackId: CallbackId, command: CommandType) => {
    wasm.wasm_command({
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

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.controllerList = new Map();
        this.socket = new Map();
    }

    public websocket_register_callback = (
        host: string,
        callback_id: CallbackId,
    ) => {
        const wasm = this.getWasm();

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
                    wasmCallback(wasm, callback_id, 'Connected');
                    return;
                }

                if (message.type === 'message') {
                    wasmCallback(wasm, callback_id, {
                        'Message': {
                            message: message.message
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
    }

    public websocket_send_message = (
        callback_id: CallbackId,
        message: string,
    ) => {
        const socket = this.socket.get(callback_id);

        if (socket === undefined) {
            console.error(`Missing socket connection for callback_id=${callback_id}`);
        } else {
            socket.send(message);
        }
    }
}
