import { BufferCursor } from './buffer_cursor';
import { jsJsonGetSize, jsJsonDecodeItem, saveJsJsonToBufferItem, JsJsonType } from './jsjson';

export interface BaseExportType {
    vertigo_export_alloc_block: (size: number) => bigint,
    vertigo_export_free_block: (pointer: bigint) => void,
    vertigo_export_wasm_callback: (callback_id: bigint, value_ptr: bigint) => bigint,
    vertigo_export_wasm_command: (value_ptr: bigint) => bigint,
};

export interface ModuleControllerType<ExportType extends BaseExportType> {
    exports: ExportType,
    getUint8Memory: () => Uint8Array,
    /**
     * @deprecated - please use wasm_command
     */
    wasm_callback: (callback_id: bigint, params: JsJsonType) => JsJsonType,
    wasm_command: (params: JsJsonType) => JsJsonType,
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

    const wasm_callback = (callback_id: bigint, value: JsJsonType): JsJsonType => {
        // Serialize JsJson directly
        const size = jsJsonGetSize(value);
        const value_ptr = exports.vertigo_export_alloc_block(size);
        const buffer = new BufferCursor(getUint8Memory, value_ptr);
        saveJsJsonToBufferItem(value, buffer);

        let result_long_ptr = exports.vertigo_export_wasm_callback(callback_id, value_ptr);

        // Decode JsJson directly
        if (result_long_ptr === 0n) {
            return null;
        }
        const resultBuffer = new BufferCursor(getUint8Memory, result_long_ptr);
        const result = jsJsonDecodeItem(resultBuffer);
        exports.vertigo_export_free_block(result_long_ptr);

        return result;
    };


    const wasm_command = (value: JsJsonType): JsJsonType => {
        // Serialize JsJson directly (no JsValue wrapper)
        const size = jsJsonGetSize(value);
        const long_ptr = exports.vertigo_export_alloc_block(size);
        const buffer = new BufferCursor(getUint8Memory, long_ptr);
        saveJsJsonToBufferItem(value, buffer);

        let result_long_ptr = exports.vertigo_export_wasm_command(long_ptr);

        // Decode JsJson directly (no JsValue wrapper)
        if (result_long_ptr === 0n) {
            return null;
        }
        const resultBuffer = new BufferCursor(getUint8Memory, result_long_ptr);
        const result = jsJsonDecodeItem(resultBuffer);
        exports.vertigo_export_free_block(result_long_ptr);

        return result;
    };


    return {
        exports,
        getUint8Memory,
        wasm_callback,
        wasm_command,
    };
};
