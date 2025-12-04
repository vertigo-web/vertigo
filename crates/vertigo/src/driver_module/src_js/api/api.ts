import { DriverWebsocket } from "./websocket/websocket";
import { assertNever } from "../assert_never";
import { JsJsonType } from "../jsjson";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";
import { fetchCacheGet } from "./command/fetchCacheGet";
import { fetchExec, FetchRequestType } from "./command/fetchExec";
import { CallbackId } from "./types";
import { Interval } from "./command/interval";
import { AppLocation } from './location/AppLocation';
import { Cookies } from "./command/cookies";
import { getRandom } from "./command/getRandom";
import { CommandType, DriverDom } from "./command/dom/dom";
import { getMetaData } from "./metadata";

type JsApiCommandType =
    | { Root: { name: string } }
    | { RootElement: { dom_id: number } }
    | { Get: { property: string } }
    | { Set: { property: string, value: JsJsonType } }
    | { Call: { method: string, args: JsJsonType[] } };

type ExecType
    = 'FetchCacheGet'
    | 'IsBrowser'
    | 'GetDateNow'
    | 'TimezoneOffset'
    | 'HistoryBack'
    | {
        FetchExec: {
            callback: CallbackId,
            request: FetchRequestType,
        }
    }
    | {
        WebsocketRegister: {
            callback: CallbackId,
            host: string
        }
    }
    | {
        WebsocketSendMessage: {
            callback: CallbackId,
            message: string,
        }
    }
    | {
        WebsocketUnregister: {
            callback: CallbackId,
        }
    }
    | {
        TimerSet: {
            callback: CallbackId,
            duration: number,
            kind: 'Interval' | 'Timeout',
        }
    }
    | {
        TimerClear: {
            callback: CallbackId,
        }
    }
    | {
        LocationGet: {
            target: 'Hash' | 'History',
        }
    }
    | {
        LocationCallback: {
            callback: number,
            mode: 'Add' | 'Remove',
            target: 'Hash' | 'History'
        }
    }
    | {
        LocationSet: {
            mode: 'Push' | 'Replace',
            target: 'Hash' | 'History'
            value: string
        }
    }
    | {
        CookieSet: {
            name: string,
            value: string,
            expires_in: number,
        }
    }
    | {
        CookieGet: {
            name: string,
        }
    }
    | {
        CookieJsonSet: {
            name: string,
            value: JsJsonType,
            expires_in: number,
        }
    }
    | {
        CookieJsonGet: {
            name: string,
        }
    }
    | {
        GetEnv: {
            name: string
        }
    }
    | {
        Log: {
            arg2: string, //"color: white; padding: 0 3px; background: green;",
            arg3: string, //"font-weight: bold; color: inherit",
            arg4: string, //"background: inherit; color: inherit",
            kind: 'Debug' | 'Info' | 'Log' | 'Warn' | 'Error',
            message: string, //"%cINFO%c crates/vertigo/src/driver_module/api/api_fetch_cache.rs:26%c FetchCache ready"
        }
    }
    | {
        GetRandom: {
            min: number,
            max: number,
        }
    }
    | {
        JsApiCall: {
            commands: Array<JsApiCommandType>
        }
    }
    | {
        DomBulkUpdate: {
            list: Array<CommandType>
        }
    };

export class Api {
    public readonly dom: DriverDom;
    private readonly websocket: DriverWebsocket;
    private readonly interval: Interval;
    private readonly location: AppLocation;
    private readonly cookie: Cookies;


    constructor(private readonly getWasm: () => ModuleControllerType<ExportType>) {
        const appLocation = new AppLocation(getWasm);

        this.dom = new DriverDom(appLocation, getWasm);
        this.websocket = new DriverWebsocket(getWasm);
        this.interval = new Interval(getWasm);
        this.location = appLocation;
        this.cookie = new Cookies();
    }

    exec(arg: JsJsonType): JsJsonType {

        //@ts-expect-error - //TODO Add safe type checking
        const safeArg: ExecType = arg;

        // console.info('exec arg', safeArg);

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

        if (safeArg === 'TimezoneOffset') {
            return {
                value: new Date().getTimezoneOffset()
            };
        }

        if (safeArg === 'HistoryBack') {
            window.history.back();
            return null;
        }

        if ('FetchExec' in safeArg) {
            fetchExec(this.getWasm, safeArg.FetchExec.callback, safeArg.FetchExec.request);
            return null;
        }

        if ('WebsocketRegister' in safeArg) {
            this.websocket.websocket_register_callback(safeArg.WebsocketRegister.host, safeArg.WebsocketRegister.callback);
            return null;
        }

        if ('WebsocketSendMessage' in safeArg) {
            this.websocket.websocket_send_message(safeArg.WebsocketSendMessage.callback, safeArg.WebsocketSendMessage.message);
            return null;
        }

        if ('WebsocketUnregister' in safeArg) {
            this.websocket.websocket_unregister_callback(safeArg.WebsocketUnregister.callback);
            return null;
        }

        if ('TimerSet' in safeArg) {
            this.interval.timerSet(safeArg.TimerSet.callback, safeArg.TimerSet.duration, safeArg.TimerSet.kind);
            return null;
        }

        if ('TimerClear' in safeArg) {
            this.interval.timerClear(safeArg.TimerClear.callback);
            return null;
        }

        if ('LocationGet' in safeArg) {
            return {
                value: this.location.get(safeArg.LocationGet.target)
            };
        }

        if ('LocationCallback' in safeArg) {
            this.location.callback(safeArg.LocationCallback.target, safeArg.LocationCallback.mode, safeArg.LocationCallback.callback);
            return null;
        }

        if ('LocationSet' in safeArg) {
            this.location.set(safeArg.LocationSet.target, safeArg.LocationSet.mode, safeArg.LocationSet.value);
            return null;
        }

        if ('CookieGet' in safeArg) {
            return {
                value: this.cookie.get(safeArg.CookieGet.name)
            };
        }

        if ('CookieSet' in safeArg) {
            this.cookie.set(safeArg.CookieSet.name, safeArg.CookieSet.value, safeArg.CookieSet.expires_in);
            return null;
        }

        if ('CookieJsonGet' in safeArg) {
            return {
                value: this.cookie.getJson(safeArg.CookieJsonGet.name)
            };
        }

        if ('CookieJsonSet' in safeArg) {
            this.cookie.setJson(safeArg.CookieJsonSet.name, safeArg.CookieJsonSet.value, safeArg.CookieJsonSet.expires_in);
            return null;
        }

        if ('GetEnv' in safeArg) {
            const name = safeArg.GetEnv.name;

            return {
                value: getMetaData(`data-env-${name}`)
            }
        }

        if ('Log' in safeArg) {
            switch (safeArg.Log.kind) {
                case 'Info': {
                    console.info(safeArg.Log.message, safeArg.Log.arg2, safeArg.Log.arg3, safeArg.Log.arg4);
                    return null;
                }
                case 'Debug': {
                    console.debug(safeArg.Log.message, safeArg.Log.arg2, safeArg.Log.arg3, safeArg.Log.arg4);
                    return null;
                }
                case 'Error': {
                    console.error(safeArg.Log.message, safeArg.Log.arg2, safeArg.Log.arg3, safeArg.Log.arg4);
                    return null;
                }
                case 'Log': {
                    console.log(safeArg.Log.message, safeArg.Log.arg2, safeArg.Log.arg3, safeArg.Log.arg4);
                    return null;
                }
                case 'Warn': {
                    console.warn(safeArg.Log.message, safeArg.Log.arg2, safeArg.Log.arg3, safeArg.Log.arg4);
                    return null;
                }
            }
        }

        if ('GetRandom' in safeArg) {
            return {
                value: getRandom(safeArg.GetRandom.min, safeArg.GetRandom.max)
            };
        }

        if ('JsApiCall' in safeArg) {
            return this.executeJsApiCall(safeArg.JsApiCall.commands);
        }

        if ('DomBulkUpdate' in safeArg) {
            this.dom.update(safeArg.DomBulkUpdate.list);
            return null;
        }

        console.info('exec_command: Arg', safeArg);
        return assertNever(safeArg);
    }

    private executeJsApiCall(commands: Array<JsApiCommandType>): JsJsonType {
        let current: any = null;

        for (const command of commands) {
            if ('Root' in command) {
                if (command.Root.name === 'window') {
                    current = window;
                } else if (command.Root.name === 'document') {
                    current = document;
                } else {
                    console.error(`Unknown root: ${command.Root.name}`);
                    return null;
                }
            } else if ('RootElement' in command) {
                const domId = command.RootElement.dom_id;
                const node = this.dom.nodes.getAnyOption(domId);
                if (node === undefined) {
                    console.error(`Element not found: ${domId}`);
                    return null;
                }
                current = node;
            } else if ('Get' in command) {
                if (current === null) {
                    console.error('Get called on null');
                    return null;
                }
                current = current[command.Get.property];
            } else if ('Set' in command) {
                if (current === null) {
                    console.error('Set called on null');
                    return null;
                }
                current[command.Set.property] = command.Set.value;
                current = undefined;
            } else if ('Call' in command) {
                if (current === null) {
                    console.error('Call called on null');
                    return null;
                }
                current = current[command.Call.method](...command.Call.args);
            }
        }

        // Convert result to JsJson
        if (current === null || current === undefined) {
            return null;
        }
        if (typeof current === 'boolean') {
            return current;
        }
        if (typeof current === 'string') {
            return current;
        }
        if (typeof current === 'number') {
            return current;
        }
        if (Array.isArray(current)) {
            return current;
        }
        if (typeof current === 'object') {
            return current;
        }
        return null;
    }
}
