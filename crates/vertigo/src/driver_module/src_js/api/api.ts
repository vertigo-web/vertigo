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
import { DriverDom } from "./command/dom/dom";
import { Metadata } from "./metadata";

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
            message: JsJsonType,
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
            callback: CallbackId,
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
        // Flat command stream, not a list of objects - see `command/dom/dom_wire.ts`.
        DomBulkUpdate: {
            commands: Uint8Array
        }
    };

// The union of `ExecType`'s object variants as [key, payload] pairs, so that switching on the
// key narrows the payload.
type EntryOf<T> = T extends object ? { [K in keyof T]-?: [K, T[K]] }[keyof T] : never;
type ExecEntry = EntryOf<Extract<ExecType, object>>;

// `console.debug`/`info`/`log`/`warn`/`error`, keyed by the Rust `ConsoleLogLevel` variant.
// Those five spellings are fixed by the wire format.
//
// Method *names*, not captured functions, and that is deliberate,
// because app can install its own console.error recorder
const CONSOLE_METHOD = {
    Debug: 'debug',
    Info: 'info',
    Log: 'log',
    Warn: 'warn',
    Error: 'error',
} as const;

export class Api {
    public readonly dom: DriverDom;
    private readonly websocket: DriverWebsocket;
    private readonly interval: Interval;
    private readonly location: AppLocation;
    private readonly cookie: Cookies;


    constructor(private readonly metadata: Metadata, private readonly getWasm: () => ModuleControllerType<ExportType>) {
        const appLocation = new AppLocation(getWasm);

        this.dom = new DriverDom(metadata, appLocation, getWasm);
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
            return fetchCacheGet(this.metadata);
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

        // Exhaustiveness check for the half of `ExecType` that is a bare string: add one
        // without a branch above and this assignment stops compiling.
        const objectArg: Extract<ExecType, object> = safeArg;

        // Every remaining variant is `{ Name: payload }` with exactly one key - that is how
        // `AutoJsJson` encodes an enum - so pulling that one entry out lets each FFI name be
        // written once, in a `case` label, instead of once per payload access.
        //
        // `?? []` is not decoration: a number or a bare string that is not handled above
        // should reach `default` and produce the same 'exec_command: Arg' line it produces
        // today, rather than throwing out of `Object.entries`.
        const [command, params] = (Object.entries(objectArg)[0] ?? []) as ExecEntry;

        switch (command) {
            case 'FetchExec':
                fetchExec(this.getWasm, params.callback, params.request);
                return null;

            case 'WebsocketRegister':
                this.websocket.websocket_register_callback(params.host, params.callback);
                return null;

            case 'WebsocketSendMessage':
                this.websocket.websocket_send_message(params.callback, params.message);
                return null;

            case 'WebsocketUnregister':
                this.websocket.websocket_unregister_callback(params.callback);
                return null;

            case 'TimerSet':
                this.interval.timerSet(params.callback, params.duration, params.kind);
                return null;

            case 'TimerClear':
                this.interval.timerClear(params.callback);
                return null;

            case 'LocationGet':
                return {
                    value: this.location.get(params.target)
                };

            case 'LocationCallback':
                this.location.callback(params.target, params.mode, params.callback);
                return null;

            case 'LocationSet':
                this.location.set(params.target, params.mode, params.value);
                return null;

            case 'CookieGet':
                return {
                    value: this.cookie.get(params.name)
                };

            case 'CookieSet':
                this.cookie.set(params.name, params.value, params.expires_in);
                return null;

            case 'CookieJsonGet':
                return {
                    value: this.cookie.getJson(params.name)
                };

            case 'CookieJsonSet':
                this.cookie.setJson(params.name, params.value, params.expires_in);
                return null;

            case 'GetEnv':
                return {
                    value: this.metadata.getEnv(params.name),
                };

            case 'Log':
                console[CONSOLE_METHOD[params.kind]](params.message, params.arg2, params.arg3, params.arg4);
                return null;

            case 'GetRandom':
                return {
                    value: getRandom(params.min, params.max)
                };

            case 'JsApiCall':
                return this.executeJsApiCall(params.commands);

            case 'DomBulkUpdate':
                this.dom.update(params.commands);
                return null;

            default:
                console.info('exec_command: Arg', safeArg);
                // Exhaustiveness check for the object half. `CommandForBrowser::SetStatus`
                // (`crates/vertigo/src/dev/command.rs`) is deliberately absent from
                // `ExecType` - `Driver::set_status` only does anything on the server - so it
                // is not checked here and, were wasm ever to send it, it would land here.
                return assertNever(command);
        }
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

        // Convert result to JsJson - sanitize host objects (Window, Element, Function, etc.)
        const isPlainObject = (obj: any): boolean => {
            if (obj === null) return false;
            if (typeof obj !== 'object') return false;
            const proto = Object.getPrototypeOf(obj);
            return proto === Object.prototype || proto === null;
        };

        const sanitize = (value: any): JsJsonType => {
            if (value === null || value === undefined) {
                return null;
            }
            if (typeof value === 'boolean') {
                return value;
            }
            if (typeof value === 'string') {
                return value;
            }
            if (typeof value === 'number') {
                return value;
            }
            if (value instanceof Uint8Array) {
                return value;
            }
            if (Array.isArray(value)) {
                return value.map((v) => sanitize(v));
            }
            if (isPlainObject(value)) {
                const out: { [k: string]: JsJsonType } = {};
                for (const k of Object.keys(value)) {
                    out[k] = sanitize(value[k]);
                }
                return out;
            }

            // Host objects (Window, Element, DOM nodes, functions, class instances, etc.)
            // are not serializable to JsJson. Return null for safety.
            return null;
        };

        return sanitize(current);
    }
}
