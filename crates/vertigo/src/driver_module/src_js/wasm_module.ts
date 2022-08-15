import { Cookies } from './module/cookies';
import { DriverDom } from './module/dom/dom';
import { DriverWebsocket } from './module/websocket/websocket';
import { Fetch } from './module/fetch';
import { HashRouter } from './module/hashrouter';
import { instant_now } from './module/instant';
import { Interval } from './module/interval';
import { wasmInit, ModuleControllerType } from './wasm_init';
import { js_call } from './js_call';
import { Guard, JsValueType } from './arguments';

//Number -> u32 or i32
//BigInt -> u64 or i64

export type ImportType = {
    panic_message: (ptr: number, length: number) => void,

    interval_set: (duration: number, callback_id: number) => number,
    interval_clear: (timer_id: number) => void,
    timeout_set: (duration: number, callback_id: number) => number,
    timeout_clear: (timer_id: number) => void,

    instant_now: () => number,

    //js_call will be replaced by dom_call, dom_get, dom_set            <-------------- TODO
    js_call: (ptr: number, size: number) => number,             //return pointer to response

    //call from rust
    dom_call: (ptr: number, size: number) => number,            //arg: dom-path, property, params: Vec<ParamItem>, return: ParamItem
    dom_get: (pth: number, size: number) => number,             //arg: dom-path, property, return ParamItem
    dom_set: (ptr: number, size: number) => void,               //arg: dom-path, property, value: ParamItem, return void
}

export type ExportType = {
    alloc: (size: number) => number,

    //call to rusta
    interval_run_callback: (callback_id: number) => void,
    timeout_run_callback: (callback_id: number) => void,
    hashrouter_hashchange_callback: (listId: number) => void,
    fetch_callback: (listId: number) => void

    websocket_callback_socket: (callback_id: number) => void;
    websocket_callback_message: (callback_id: number) => void;
    websocket_callback_close: (callback_id: number) => void;

    dom_mousedown: (dom_id: bigint) => void,
    dom_mouseover: (dom_id: bigint) => void;
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
                    (ptr: number, size: number): JsValueType => getWasm().decodeArguments(ptr, size),
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

                dom_call: (ptr: number, size: number): number => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const [domPath, property, params, ...rest] = args;

                        if (Array.isArray(domPath) && Guard.isString(property) && Array.isArray(params) && rest.length === 0) {
                            const response = dom.dom_call(domPath, property, params);
                            return getWasm().newList().saveListItem(response);
                        }
                    }

                    console.error('dom_call - wrong parameters', args);
                    return 0;
                },
                dom_get: (ptr: number, size: number): number => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const [domPath, property, ...rest] = args;

                        if (Array.isArray(domPath) && Guard.isString(property) && rest.length === 0) {
                            const response = dom.dom_get(domPath, property);
                            return getWasm().newList().saveListItem(response);
                        }
                    }

                    console.error('dom_get - wrong parameters', args);
                    return 0;
                },
                dom_set: (ptr: number, size: number): void => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const [domPath, property, value, ...rest] = args;

                        if (Array.isArray(domPath) && Guard.isString(property) && rest.length === 0) {
                            dom.dom_set(domPath, property, value);
                            return;
                        }
                    }

                    console.error('dom_set - wrong parameters', args);
                }
            }
        });

        return new WasmModule(wasmModule);
    }
}
