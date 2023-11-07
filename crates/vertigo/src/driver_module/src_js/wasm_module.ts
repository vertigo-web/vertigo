import { wasmInit, ModuleControllerType } from './wasm_init';
import { ApiBrowser as ApiBrowser } from './api_browser';
import { JsNode } from './js_node';

//Number -> u32 or i32
//BigInt -> u64 or i64

export type ImportType = {
    panic_message: (ptr: number, length: number) => void,
    //call from rust
    dom_access: (ptr: number, size: number) => number,
}

export type ExportType = {
    alloc: (size: number) => number,
    free: (pointer: number) => void,
    wasm_callback: (callback_id: bigint, value_ptr: number) => bigint,  //result => pointer: 32bit, size: 32bit
    vertigo_entry_function: (major: number, minor: number) => void,
}

export class WasmModule {
    private readonly wasm: ModuleControllerType<ExportType>;

    private constructor(
        wasm: ModuleControllerType<ExportType>,
    ) {
        this.wasm = wasm;
    }

    public vertigo_entry_function(major: number, minor: number) {
        this.wasm.exports.vertigo_entry_function(major, minor);
    }

    public static async create(wasmBinPath: string): Promise<WasmModule> {
        let wasmModule: ModuleControllerType<ExportType> | null = null;

        const getWasm = (): ModuleControllerType<ExportType> => {
            if (wasmModule === null) {
                throw Error('Wasm is no initialized');
            }

            return wasmModule;
        };

        const apiBrowser = new ApiBrowser(getWasm);

        //@ts-expect-error
        window.$vertigoApi = apiBrowser;

        wasmModule = await wasmInit<ImportType, ExportType>(wasmBinPath, {
            mod: {
                panic_message: (ptr: number, size: number) => {
                    const decoder = new TextDecoder("utf-8");
                    const m = getWasm().getUint8Memory().subarray(ptr, ptr + size);
                    const message = decoder.decode(m);
                    console.error('PANIC', message);
                },
                dom_access: (ptr: number, size: number): number => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const path = args;
                        let wsk = new JsNode(apiBrowser, apiBrowser.dom.nodes, null);

                        for (const pathItem of path) {
                            const newWsk = wsk.next(path, pathItem);

                            if (newWsk === null) {
                                return 0;
                            }

                            wsk = newWsk;
                        }

                        return getWasm().valueSaveToBuffer(wsk.toValue());
                    }

                    console.error('dom_access - wrong parameters', args);
                    return 0;
                },
            }
        });

        return new WasmModule(wasmModule);
    }
}
