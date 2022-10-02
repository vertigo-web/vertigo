import { argumentsDecode, JsValueType, JsValueBuilder, saveToBuffer } from './arguments';

export interface BaseExportType {
    alloc: (size: number) => number,
};

export interface ModuleControllerType<ExportType extends BaseExportType> {
    exports: ExportType,
    decodeArguments: (ptr: number, size: number) => JsValueType,
    newList: () => JsValueBuilder,
    getUint8Memory: () => Uint8Array,
    saveJsValue: (value: JsValueType) => number,
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

    const decodeArguments = (ptr: number, size: number) => argumentsDecode(getUint8Memory, ptr, size);

    const newList = (): JsValueBuilder => new JsValueBuilder(getUint8Memory, exports.alloc);

    const saveJsValue = (value: JsValueType): number => saveToBuffer(getUint8Memory, exports.alloc, value);

    return {
        exports,
        decodeArguments,
        getUint8Memory,
        newList,
        saveJsValue
    };
};
