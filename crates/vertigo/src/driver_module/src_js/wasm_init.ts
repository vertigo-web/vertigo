import { argumentsDecode, ListItemType, ParamListBuilder } from "./arguments";

export interface BaseExportType {
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
};

export interface ModuleControllerType<ExportType extends BaseExportType> {
    exports: ExportType,
    decodeArguments: (ptr: number) => ListItemType,
    newList: () => ParamListBuilder,
    getUint8Memory: () => Uint8Array,
}

const fetchModule = async (wasmBinPath: string, imports: Record<string, WebAssembly.ModuleImports>): Promise<WebAssembly.WebAssemblyInstantiatedSource> => {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
        console.info('fetchModule by WebAssembly.instantiateStreaming');

        try {
            const module = await WebAssembly.instantiateStreaming(fetch(wasmBinPath), imports);
            return module;
        } catch (err) {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", err);
        }
    }

    console.info('fetchModule by WebAssembly.instantiate');

    const resp = await fetch(wasmBinPath);
    const binary = await resp.arrayBuffer();
    const module_instance = await WebAssembly.instantiate(binary, imports);
    return module_instance;
};

export const wasmInit = async <ImportType extends Record<string, Function>, ExportType extends BaseExportType>(
    wasmBinPath: string,
    imports: { mod: ImportType },
): Promise<ModuleControllerType<ExportType>> => {
    const module_instance = await fetchModule(wasmBinPath, imports);

    let cachegetUint8Memory: Uint8Array = new Uint8Array(1);

    const getUint8Memory = () => {
        if (module_instance.instance.exports.memory instanceof WebAssembly.Memory) {
            if (cachegetUint8Memory.buffer !== module_instance.instance.exports.memory.buffer) {
                console.info('getUint8Memory: reallocate the Uint8Array for a new size', module_instance.instance.exports.memory.buffer.byteLength);
                cachegetUint8Memory = new Uint8Array(module_instance.instance.exports.memory.buffer);
            }
            return cachegetUint8Memory;
        } else {
            throw Error('Missing memory');
        }
    };

    //@ts-expect-error
    const exports: ExportType = module_instance.instance.exports;

    const decodeArguments = (ptr: number) => argumentsDecode(getUint8Memory, ptr);

    const newList = (): ParamListBuilder => new ParamListBuilder(getUint8Memory, exports);

    return {
        exports,
        decodeArguments,
        getUint8Memory,
        newList
    };
};
