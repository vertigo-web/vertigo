import { Cookies } from './module/cookies';
import { DriverDom } from './module/dom/dom';
import { DriverWebsocket } from './module/websocket/websocket';
import { Fetch } from './module/fetch';
import { HashRouter } from './module/hashrouter';
import { instant_now } from './module/instant';
import { Interval } from './module/interval';
import { wasmInit, ModuleControllerType } from './wasm_init';

//Number -> u32 or i32
//BigInt -> u64 or i64

type Console1Type = (
    arg1_ptr: BigInt, arg1_len: BigInt,
) => void;

type Console4Type = (
    arg1_ptr: BigInt, arg1_len: BigInt,
    arg2_ptr: BigInt, arg2_len: BigInt,
    arg3_ptr: BigInt, arg3_len: BigInt,
    arg4_ptr: BigInt, arg4_len: BigInt,
) => void;

export type ImportType = {
    //call from rust

    console_error_1: Console1Type,
    console_debug_4: Console4Type,
    console_log_4: Console4Type,
    console_info_4: Console4Type,
    console_warn_4: Console4Type,
    console_error_4: Console4Type,

    cookie_get: (cname_ptr: BigInt, cname_len: BigInt) => void,
    cookie_set: (
        cname_ptr: BigInt, cname_len: BigInt,
        cvalue_ptr: BigInt, cvalue_len: BigInt,
        expires_in: BigInt,
    ) => void,
    interval_set: (duration: number, callback_id: number) => number,
    interval_clear: (timer_id: number) => void,
    timeout_set: (duration: number, callback_id: number) => number,
    timeout_clear: (timer_id: number) => void,

    instant_now: () => number,

    hashrouter_get_hash_location: () => void,                                                   //return from js on the stack with the current hash
    hashrouter_push_hash_location: (new_hash_ptr: BigInt, new_hash_length: BigInt) => void,

    fetch_send_request: (
        request_id: number,
        method_ptr: BigInt,
        method_len: BigInt,
        url_ptr: BigInt,
        url_len: BigInt,
        headers_ptr: BigInt,
        headers_len: BigInt,
        body_ptr: BigInt,
        body_len: BigInt,
    ) => void,

    websocket_register_callback: (host_ptr: BigInt, host_len: BigInt, callback_id: number) => void,
    websocket_unregister_callback: (callback_id: number) => void,
    websocket_send_message: (callback_id: number, message_ptr: BigInt, message_len: BigInt) => void,

    dom_bulk_update: (value_ptr: BigInt, value_len: BigInt) => void,
    dom_get_bounding_client_rect_x: (id: BigInt) => number;
    dom_get_bounding_client_rect_y: (id: BigInt) => number;
    dom_get_bounding_client_rect_width: (id: BigInt) => number;
    dom_get_bounding_client_rect_height: (id: BigInt) => number;
    dom_scroll_top: (node_id: BigInt) => number;
    dom_set_scroll_top: (node_id: BigInt, value: number) => void;
    dom_scroll_left: (node_id: BigInt) => number;
    dom_set_scroll_left: (node_id: BigInt, value: number) => void;
    dom_scroll_width: (node_id: BigInt) => number;
    dom_scroll_height: (node_id: BigInt) => number;
}

export type ExportType = {
    alloc_empty_string: () => void,
    alloc: (length: BigInt) => BigInt,

    //call to rusta
    interval_run_callback: (callback_id: number) => void,
    timeout_run_callback: (callback_id: number) => void,
    hashrouter_hashchange_callback: () => void,                                                 //Parameter for rust on the stack: current hash
    fetch_callback: (request_id: number, success: number, status: number) => void               //Parameter for rust on the stack: response

    websocket_callback_socket: (callback_id: number) => void;
    websocket_callback_message: (callback_id: number) => void;                                  //Parameter for rust on the stack: message
    websocket_callback_close: (callback_id: number) => void;

    dom_mousedown: (dom_id: BigInt) => void,
    dom_mouseover: (
        dom_id: BigInt                                                                          //0 - null
    ) => void;
    dom_keydown: (                                                                              //Parameter for rust on the stack: key: string, code: string,
        dom_id: BigInt,                                                                         // 0 - null
        alt_key: number,                                                                        // 0 - false, >0 - true
        ctrl_key: number,                                                                       // 0 - false, >0 - true
        shift_key: number,                                                                      // 0 - false, >0 - true
        meta_key: number                                                                        // 0 - false, >0 - true
    ) => number;                                                                                // 0 - false, >0 - true
    dom_oninput: (dom_id: BigInt) => void,                                                      //Parameter for rust on the stack: text: string

    start_application: () => void,
}


export class WasmModule {
    private readonly wasm: ModuleControllerType<ExportType>;

    private constructor(
        wasm: ModuleControllerType<ExportType>,
    ) {
        this.wasm = wasm;
    }

    public start_application() {
        this.wasm.exports.start_application();
    }

    public static async create(wasmBinPath: string): Promise<WasmModule> {

        let wasmModule: ModuleControllerType<ExportType> | null = null;

        const getWasm = (): ModuleControllerType<ExportType> => {
            if (wasmModule === null) {
                throw Error('Wasm is no initialized');
            }

            return wasmModule;
        }

        const cookies = new Cookies(getWasm);
        const interval = new Interval(getWasm);
        const hashRouter = new HashRouter(getWasm);
        const fetchModule = new Fetch(getWasm);
        const websocket = new DriverWebsocket(getWasm);
        const dom = new DriverDom(getWasm);

        const console_error_1 = (
            arg1_ptr: BigInt, arg1_len: BigInt,
        ) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            console.error(arg1);
        };

        const console_debug_4 = (
            arg1_ptr: BigInt, arg1_len: BigInt,
            arg2_ptr: BigInt, arg2_len: BigInt,
            arg3_ptr: BigInt, arg3_len: BigInt,
            arg4_ptr: BigInt, arg4_len: BigInt
        ) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.debug(arg1, arg2, arg3, arg4);
        };

        const console_log_4 = (
            arg1_ptr: BigInt, arg1_len: BigInt,
            arg2_ptr: BigInt, arg2_len: BigInt,
            arg3_ptr: BigInt, arg3_len: BigInt,
            arg4_ptr: BigInt, arg4_len: BigInt
        ) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.log(arg1, arg2, arg3, arg4);
        };

        const console_info_4 = (
            arg1_ptr: BigInt, arg1_len: BigInt,
            arg2_ptr: BigInt, arg2_len: BigInt,
            arg3_ptr: BigInt, arg3_len: BigInt,
            arg4_ptr: BigInt, arg4_len: BigInt
        ) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.info(arg1, arg2, arg3, arg4);
        };

        const console_warn_4 = (
            arg1_ptr: BigInt, arg1_len: BigInt,
            arg2_ptr: BigInt, arg2_len: BigInt,
            arg3_ptr: BigInt, arg3_len: BigInt,
            arg4_ptr: BigInt, arg4_len: BigInt
        ) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.warn(arg1, arg2, arg3, arg4);
        };

        const console_error_4 = (
            arg1_ptr: BigInt, arg1_len: BigInt,
            arg2_ptr: BigInt, arg2_len: BigInt,
            arg3_ptr: BigInt, arg3_len: BigInt,
            arg4_ptr: BigInt, arg4_len: BigInt
        ) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.error(arg1, arg2, arg3, arg4);
        };

        wasmModule = await wasmInit<ImportType, ExportType>(wasmBinPath, {
            mod: {
                console_error_1,
                console_debug_4,
                console_log_4,
                console_info_4,
                console_warn_4,
                console_error_4,
                cookie_get: cookies.get,
                cookie_set: cookies.set,
                interval_set: interval.interval_set,
                interval_clear: interval.interval_clear,
                timeout_set: interval.timeout_set,
                timeout_clear: interval.timeout_clear,
                instant_now,
                hashrouter_get_hash_location: hashRouter.hashrouter_get_hash_location,
                hashrouter_push_hash_location: hashRouter.hashrouter_push_hash_location,
                fetch_send_request: fetchModule.fetch_send_request,
                websocket_register_callback: websocket.websocket_register_callback,
                websocket_unregister_callback: websocket.websocket_unregister_callback,
                websocket_send_message: websocket.websocket_send_message,
                dom_bulk_update: dom.dom_bulk_update,
                dom_get_bounding_client_rect_x: dom.dom_get_bounding_client_rect_x,
                dom_get_bounding_client_rect_y: dom.dom_get_bounding_client_rect_y,
                dom_get_bounding_client_rect_width: dom.dom_get_bounding_client_rect_width,
                dom_get_bounding_client_rect_height: dom.dom_get_bounding_client_rect_height,
                dom_scroll_top: dom.dom_scroll_top,
                dom_set_scroll_top: dom.dom_set_scroll_top,
                dom_scroll_left: dom.dom_scroll_left,
                dom_set_scroll_left: dom.dom_set_scroll_left,
                dom_scroll_width: dom.dom_scroll_width,
                dom_scroll_height: dom.dom_scroll_height,
            }
        });

        return new WasmModule(wasmModule);
    }
}
