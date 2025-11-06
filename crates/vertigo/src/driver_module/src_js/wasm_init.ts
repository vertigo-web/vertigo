import { jsValueDecode, saveToBuffer, saveToBufferLongPtr } from './jsvalue';
import { JsValueType } from './jsvalue_types';

export interface BaseExportType {
    alloc: (size: number) => number,
    free: (pointer: number) => void,
    wasm_callback: (callback_id: bigint, value_ptr: bigint) => bigint,   //result => pointer: 32bit, size: 32bit
};

export interface ModuleControllerType<ExportType extends BaseExportType> {
    exports: ExportType,
    decodeArgumentsLong: (long_ptr: bigint) => JsValueType,
    getUint8Memory: () => Uint8Array,
    wasm_callback: (callback_id: bigint, params: JsValueType) => JsValueType,
    valueSaveToBuffer: (value: JsValueType) => number,
    valueSaveToBufferLong: (value: JsValueType) => bigint,
}

const fetchModule = async (wasmBinPath: string, imports: Record<string, WebAssembly.ModuleImports>): Promise<WebAssembly.WebAssemblyInstantiatedSource> => {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
        const stream = fetch(wasmBinPath);
        try {
            const module = await WebAssembly.instantiateStreaming(stream, imports);
            return module;
        } catch (err) {
            console.warn("`WebAssembly.instantiateStreaming` failed. This could happen if your server does not serve wasm with `application/wasm` MIME type, but check the original error too. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", err);
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

    let cacheGetUint8Memory: Uint8Array = new Uint8Array(1);

    const getUint8Memory = () => {
        if (module_instance.instance.exports.memory instanceof WebAssembly.Memory) {
            if (cacheGetUint8Memory.buffer !== module_instance.instance.exports.memory.buffer) {
                cacheGetUint8Memory = new Uint8Array(module_instance.instance.exports.memory.buffer);
            }
            return cacheGetUint8Memory;
        } else {
            throw Error('Missing memory');
        }
    };

    //@ts-expect-error
    const exports: ExportType = module_instance.instance.exports;

    const decodeArgumentsLong = (long_ptr: bigint): JsValueType => {
        if (long_ptr === 0n) {
            return undefined;
        }

        const size = Number(long_ptr % (2n ** 32n));
        const ptr = Number(long_ptr >> 32n);

        const response = jsValueDecode(getUint8Memory, ptr, size);
        exports.free(Number(ptr));

        return response;
    };

    const valueSaveToBuffer = (value: JsValueType): number => saveToBuffer(getUint8Memory, exports.alloc, value);
    const valueSaveToBufferLong = (value: JsValueType): bigint => saveToBufferLongPtr(getUint8Memory, exports.alloc, value);

    const wasm_callback = (callback_id: bigint, value: JsValueType): JsValueType => {
        const value_ptr = valueSaveToBufferLong(value);
        let result_long_ptr = exports.wasm_callback(callback_id, value_ptr);
        return decodeArgumentsLong(result_long_ptr);
    };

    return {
        exports,
        decodeArgumentsLong,
        getUint8Memory,
        wasm_callback,
        valueSaveToBuffer,
        valueSaveToBufferLong,
    };
};
