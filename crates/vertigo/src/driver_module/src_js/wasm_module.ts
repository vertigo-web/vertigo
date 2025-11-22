import { wasmInit, ModuleControllerType } from './wasm_init';
import { JsNode } from './js_node';
import { GuardJsValue } from './guard';
import { ExecCommand } from './exec_command/exec_command';
import { JsValueConst } from './jsvalue_types';

//Number -> u32 or i32
//BigInt -> u64 or i64

export type ImportType = {
    panic_message: (long_ptr: bigint) => void,
    //call from rust - To be deleted eventually
    dom_access: (long_ptr: bigint) => bigint,
}

export type ExportType = {
    vertigo_export_alloc_block: (size: number) => bigint,
    vertigo_export_free_block: (pointer: bigint) => void,
    //TODO - This function is to be removed eventually.
    vertigo_export_wasm_callback: (callback_id: bigint, value_ptr: bigint) => bigint,
    vertigo_export_wasm_command: (value_ptr: bigint) => bigint,
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

        const execCommand = new ExecCommand(getWasm);

        //@ts-expect-error
        window.$vertigoApi = execCommand;

        wasmModule = await wasmInit<ImportType, ExportType>(wasmBinPath, {
            mod: {
                panic_message: (long_ptr: bigint) => {

                    const size = Number(long_ptr % (2n ** 32n));
                    const ptr = Number(long_ptr >> 32n);

                    const decoder = new TextDecoder("utf-8");
                    const m = getWasm().getUint8Memory().subarray(ptr, ptr + size);
                    const message = decoder.decode(m);
                    console.error('PANIC', message);
                },
                dom_access: (long_ptr: bigint): bigint => {
                    let args = getWasm().decodeArgumentsLong(long_ptr);

                    //new wersion
                    if (GuardJsValue.isJson(args)) {
                        const response = execCommand.exec(args.value);
                        return getWasm().valueSaveToBufferLong({
                            type: JsValueConst.Json,
                            value: response
                        });
                    }

                    //old version
                    if (Array.isArray(args)) {
                        const path = args;
                        let wsk = new JsNode(execCommand.dom.nodes, null);

                        for (const pathItem of path) {
                            const newWsk = wsk.next(path, pathItem);

                            if (newWsk === null) {
                                return 0n;
                            }

                            wsk = newWsk;
                        }

                        return getWasm().valueSaveToBufferLong(wsk.toValue());
                    }

                    console.error('dom_access - wrong parameters', args);
                    return 0n;
                }
            }
        });

        return new WasmModule(wasmModule);
    }
}
