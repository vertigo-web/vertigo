import { Guard, ListItemType } from "./arguments";
import { consoleLog } from "./js_call/consoleLog";
import { initCookie } from "./js_call/cookie";
import { initDom } from "./js_call/dom";
import { initFetch } from "./js_call/fetch";
import { initHashrouter } from "./js_call/hashrouter";
import { initWebsocketModule } from "./js_call/websocket";
import { Cookies } from "./module/cookies";
import { DriverDom } from "./module/dom/dom";
import { Fetch } from "./module/fetch";
import { HashRouter } from "./module/hashrouter";
import { DriverWebsocket } from "./module/websocket/websocket";
import { ModuleControllerType } from "./wasm_init";
import { ExportType } from "./wasm_module";

export const js_call = (
    decodeArguments: (ptr: number) => ListItemType,
    getWasm: () => ModuleControllerType<ExportType>,
    fetch: Fetch,
    cookies: Cookies,
    dom: DriverDom,
    hashRouter: HashRouter,
    websocket: DriverWebsocket,
) => {

    const modules: Record<string, (method: string, arg: Array<ListItemType>) => number> = {
        consoleLog: consoleLog,
        hashrouter: initHashrouter(getWasm, hashRouter),
        websocket: initWebsocketModule(websocket),
        cookie: initCookie(getWasm, cookies),
        fetch: initFetch(fetch),
        dom: initDom(dom),
    }

    return (ptr: number): number => {
        const args = decodeArguments(ptr);

        // console.info('js_call', arg);        //for debug

        if (Array.isArray(args)) {
            const [modulePrefix, moduleName, newFunction, ...restNew] = args;

            if (
                modulePrefix === 'module' &&
                Guard.isString(moduleName) &&
                Guard.isString(newFunction)
            ) {
                const toRun = modules[moduleName];

                if (toRun === undefined) {
                    console.error(`js-call: unknown module ${moduleName}`, args);
                    return 0;
                }

                return toRun(newFunction, restNew);
            }

            console.error('js-call: incorrect parameters', args);
            return 0;    
        }

        console.error("js-call: List of parameters was expected: ", args);
        return 0;
    };
};
