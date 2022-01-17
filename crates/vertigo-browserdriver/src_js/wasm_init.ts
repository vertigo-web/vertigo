export interface BaseExportType {
    alloc: (length: BigInt) => BigInt,
    alloc_empty_string: () => void,
};

export interface ModuleControllerType<ExportType extends BaseExportType> {
    exports: ExportType,
    decodeText: (ptr: BigInt, length: BigInt) => string,
    decodeTextNull: (ptr: BigInt, length: BigInt) => string | null,
    pushString: (value: string) => void,
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


    const decodeTextNull = (ptr: BigInt, length: BigInt): string | null => {
        if (length === 0n) {
            return null;
        }

        const m = getUint8Memory().subarray(Number(ptr), Number(ptr) + Number(length));
        var decoder = new TextDecoder("utf-8");
        return decoder.decode(m.slice(0, Number(length)));
    };

    const decodeText = (ptr: BigInt, length: BigInt): string => {
        if (length === 0n) {
            return '';
        }

        const m = getUint8Memory().subarray(Number(ptr), Number(ptr) + Number(length));
        var decoder = new TextDecoder("utf-8");
        return decoder.decode(m.slice(0, Number(length)));
    };

    //@ts-expect-error
    const exports: ExportType = module_instance.instance.exports;

    const cachedTextEncoder = new TextEncoder();

    const pushString = (arg: string) => {
        if (arg.length === 0) {
            exports.alloc_empty_string();
            return;
        }

        const buf = cachedTextEncoder.encode(arg);
        const ptr = Number(exports.alloc(BigInt(buf.length)));

        getUint8Memory().subarray(ptr, ptr + buf.length).set(buf);
    };

    return {
        exports,
        decodeText,
        decodeTextNull,
        pushString
    };
};
