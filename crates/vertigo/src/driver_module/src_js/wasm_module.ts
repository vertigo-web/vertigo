import { Cookies } from './module/cookies';
import { DriverDom } from './module/dom/dom';
import { DriverWebsocket } from './module/websocket/websocket';
import { Fetch } from './module/fetch';
import { HashRouter } from './module/hashrouter';
import { instant_now } from './module/instant';
import { Interval } from './module/interval';
import { wasmInit, ModuleControllerType } from './wasm_init';
import { js_call } from './js_call';
import { ListItemType } from './arguments';

//Number -> u32 or i32
//BigInt -> u64 or i64

export type ImportType = {
    //call from rust
    js_call: (params: number) => number,
    panic_message: (ptr: number, length: number) => void,

    interval_set: (duration: number, callback_id: number) => number,
    interval_clear: (timer_id: number) => void,
    timeout_set: (duration: number, callback_id: number) => number,
    timeout_clear: (timer_id: number) => void,

    instant_now: () => number,
}

export type ExportType = {
    arguments_debug: (listId: number) => void,                                  //fn(u32)
    arguments_new_list: () => number,                                           //fn() -> u32
    arguments_push_string_empty: (listId: number) => void,                      //fn(u32) -> u32
    arguments_push_string_alloc: (listId: number, size: number) => number,      //fn(u32, u32) -> u32
    arguments_push_buffer_alloc: (listId: number, size: number) => number,      //fn(u32, u32) -> u32
    arguments_push_u32: (listId: number, value: number) => void,                //fn(u32, u32)
    arguments_push_i32: (listId: number, value: number) => void,                //fn(u32, i32)
    arguments_push_u64: (listId: number, value: BigInt) => void,                //fn(u32, u64)
    arguments_push_i64: (listId: number, value: BigInt) => void,                //fn(u32, i64)
    arguments_push_true: (listId: number) => void,                              //fn(u32)
    arguments_push_false: (listId: number) => void,                             //fn(u32)
    arguments_push_null: (listId: number) => void,                              //fn(u32)
    arguments_push_sublist: (paramsId: number, sub_params_id: number) => void,  //fn(u32, u32)
    arguments_freeze: (listId: number) => void,                                  //fn(u32)

    //call to rusta
    interval_run_callback: (callback_id: number) => void,
    timeout_run_callback: (callback_id: number) => void,
    hashrouter_hashchange_callback: (listId: number) => void,
    fetch_callback: (listId: number) => void

    websocket_callback_socket: (callback_id: number) => void;
    websocket_callback_message: (callback_id: number) => void;
    websocket_callback_close: (callback_id: number) => void;

    dom_mousedown: (dom_id: BigInt) => void,
    dom_mouseover: (dom_id: BigInt) => void;
    dom_keydown: (params_id: number) => number;       // 0 - false, >0 - true
    dom_oninput: (params_id: number) => void,
    dom_ondropfile: (params_id: number) => void,

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

        const cookies = new Cookies();
        const interval = new Interval(getWasm);
        const hashRouter = new HashRouter(getWasm);
        const fetchModule = new Fetch(getWasm);
        const websocket = new DriverWebsocket(getWasm);
        const dom = new DriverDom(getWasm);

        wasmModule = await wasmInit<ImportType, ExportType>(wasmBinPath, {
            mod: {
                panic_message: (ptr: number, size: number) => {
                    const decoder = new TextDecoder("utf-8");
                    const m = getWasm().getUint8Memory().subarray(ptr, ptr + size);
                    const message = decoder.decode(m);
                    console.error('PANIC', message);
                },
                js_call: js_call(
                    (ptr: number): ListItemType => getWasm().decodeArguments(ptr),
                    getWasm,
                    fetchModule,
                    cookies,
                    dom,
                    hashRouter,
                    websocket,
                ),
                interval_set: interval.interval_set,
                interval_clear: interval.interval_clear,
                timeout_set: interval.timeout_set,
                timeout_clear: interval.timeout_clear,
                instant_now,
            }
        });

        return new WasmModule(wasmModule);
    }
}
