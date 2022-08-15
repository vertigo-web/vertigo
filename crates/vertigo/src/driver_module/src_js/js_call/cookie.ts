import { Guard, JsValueType } from "../arguments";
import { Cookies } from "../module/cookies";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export const initCookie = (getWasm: () => ModuleControllerType<ExportType>, cookies: Cookies) => (method: string, args: Array<JsValueType>): number => {
    if (method === 'get') {
        const [name, ...rest] = args;

        if (Guard.isString(name) && rest.length === 0) {
            const value = cookies.get(name);

            const params = getWasm().newList();
            params.push_string(value);
            return params.saveToBuffer();
        }

        console.error('js-call -> module -> cookie -> get: incorrect parameters', args);
        return 0;
    }

    if (method === 'set') {
        const [name, value, expires_in, ...rest] = args;

        if (
            Guard.isString(name) &&
            Guard.isString(value) &&
            Guard.isBigInt(expires_in) &&
            rest.length === 0
        ) {
            cookies.set(name, value, expires_in.value);
            return 0;
        }

        console.error('js-call -> module -> cookie -> set: incorrect parameters', args);
        return 0;
    }

    console.error('js-call -> module -> cookie: incorrect parameters', args);
    return 0;
};
