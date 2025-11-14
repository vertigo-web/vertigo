import { assertNever } from "../assert_never";
import { JsJsonType } from "../jsjson";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";
import { fetchCacheGet } from "./command/fetchCacheGet";
import { fetchExec, FetchRequestType } from "./command/fetchExec";
import { CallbackId } from "./types";

type ExecType
    = 'FetchCacheGet'
    | 'IsBrowser'
    | 'GetDateNow'
    | {
        'FetchExec': {
            callback: CallbackId,
            request: FetchRequestType,
        }
    };


export const exec_command = (getWasm: () => ModuleControllerType<ExportType>, arg: JsJsonType): JsJsonType => {

    //@ts-expect-error - //TODO Add safe type checking
    const safeArg: ExecType = arg;

    if (safeArg === 'FetchCacheGet') {
        return fetchCacheGet();
    }

    if (safeArg === 'IsBrowser') {
        return {
            value: true
        };
    }

    if (safeArg === 'GetDateNow') {
        return {
            value: Date.now(),
        };
    }

    if ('FetchExec' in safeArg) {
        fetchExec(getWasm, safeArg.FetchExec.callback, safeArg.FetchExec.request);
        return null;
    }

    console.info('exec_command: Arg', safeArg);
    return assertNever(safeArg);
};

