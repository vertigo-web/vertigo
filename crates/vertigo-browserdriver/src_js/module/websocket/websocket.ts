import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { SocketConnection, SocketConnectionController } from "./connection";


const assertNeverMessage = (data: never): never => {
    console.error(data);
    throw Error('unknown message');
};

export class DriverWebsocket {
    private getWasm: () => ModuleControllerType<ExportType>;
    private readonly controllerList: Map<number, SocketConnectionController>;
    private readonly socket: Map<number, SocketConnection>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.controllerList = new Map();
        this.socket = new Map();
    }

    public websocket_register_callback = (
        host_ptr: BigInt,   //string,
        host_len: BigInt,
        callback_id: number,
    ) => {
        const wasm = this.getWasm();
        const host = wasm.decodeText(host_ptr, host_len);

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
                    wasm.exports.websocket_callback_socket(callback_id);
                    return;
                }
        
                if (message.type === 'message') {
                    wasm.pushString(message.message);
                    wasm.exports.websocket_callback_message(callback_id);
                    return;
                }

                if (message.type === 'close') {
                    wasm.exports.websocket_callback_close(callback_id);
                    this.socket.delete(callback_id);
                    return;
                }

                return assertNeverMessage(message);
            }
        );

        this.controllerList.set(callback_id, controller);
    }

    public websocket_unregister_callback = (callback_id: number) => {
        const controller = this.controllerList.get(callback_id);

        if (controller === undefined) {
            console.error('Expected controller');
            return;
        }

        controller.dispose();
        this.controllerList.delete(callback_id);
    }

    public websocket_send_message = (
        callback_id: number,
        message_ptr: BigInt, //string,
        message_len: BigInt,
    ) => {
        const message = this.getWasm().decodeText(message_ptr, message_len);
        const socket = this.socket.get(callback_id);

        if (socket === undefined) {
            console.error(`Missing socket connection for callback_id=${callback_id}`);
        } else {
            socket.send(message);
        }
    }
}
