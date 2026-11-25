import { BufferCursor } from './buffer_cursor';
import { jsJsonGetSize, jsJsonDecodeItem, saveJsJsonToBufferItem, JsJsonType } from './jsjson';

export interface BaseExportType {
    vertigo_export_alloc_block: (size: number) => bigint,
    vertigo_export_free_block: (pointer: bigint) => void,
    vertigo_export_wasm_command: (value_ptr: bigint) => bigint,
};

export interface ModuleControllerType<ExportType extends BaseExportType> {
    exports: ExportType,
    getUint8Memory: () => Uint8Array,
    wasmCommand: (params: JsJsonType) => JsJsonType,
}

// The wasm as the bootstrap began to download it, ahead of this code (see `boot.ts`).
export interface WasmDownload {
    // Compiled while the page was loading (`vertigo serve --wasm-preload`), `null` if that failed.
    module?: Promise<WebAssembly.Module | null>,
    // The answer to the request made once the page had loaded, along with the one for this code.
    response?: Promise<Response>,
}

const fetchModule = async (
    wasmBinPath: string,
    imports: Record<string, WebAssembly.ModuleImports>,
    download: WasmDownload,
): Promise<WebAssembly.WebAssemblyInstantiatedSource> => {
    const preloaded = await download.module;
    if (preloaded) {
        return {
            module: preloaded,
            instance: await WebAssembly.instantiate(preloaded, imports),
        };
    }

    let response = download.response;

    if (typeof WebAssembly.instantiateStreaming === 'function') {
        const stream = response ?? fetch(wasmBinPath);
        // Read by the streaming, whether it fails or not
        response = undefined;
        try {
            const module = await WebAssembly.instantiateStreaming(stream, imports);
            return module;
        } catch (err) {
            console.warn("`WebAssembly.instantiateStreaming` failed. This could happen if your server does not serve wasm with `application/wasm` MIME type, but check the original error too. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", err);
        }
    }

    console.info('fetchModule by WebAssembly.instantiate');

    const resp = await (response ?? fetch(wasmBinPath));
    const binary = await resp.arrayBuffer();
    const module_instance = await WebAssembly.instantiate(binary, imports);
    return module_instance;
};

export const wasmInit = async <ExportType extends BaseExportType>(
    wasmBinPath: string,
    imports: { mod: WebAssembly.ModuleImports },
    download: WasmDownload,
): Promise<ModuleControllerType<ExportType>> => {
    const module_instance = await fetchModule(wasmBinPath, imports, download);

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

    //@ts-expect-error the exports the wasm was built with, which its types know nothing of
    const exports: ExportType = module_instance.instance.exports;

    const wasmCommand = (value: JsJsonType): JsJsonType => {
        // Serialize JsJson
        const size = jsJsonGetSize(value);
        const long_ptr = exports.vertigo_export_alloc_block(size);
        const buffer = new BufferCursor(getUint8Memory, long_ptr);
        saveJsJsonToBufferItem(value, buffer);

        const result_long_ptr = exports.vertigo_export_wasm_command(long_ptr);

        // Decode JsJson
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
        wasmCommand: wasmCommand,
    };
};
