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

type ExecType
    = 'FetchCacheGet'
    | 'IsBrowser'
    | 'GetDateNow'
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
    };

export class ExecCommand {
    private readonly websocket: DriverWebsocket;
    private readonly interval: Interval;
    private readonly location: AppLocation;
    private readonly cookie: Cookies;
    

    constructor(private readonly getWasm: () => ModuleControllerType<ExportType>, location: AppLocation) {
        this.websocket = new DriverWebsocket(getWasm);
        this.interval = new Interval(getWasm);
        this.location = location;
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
                value: this.cookie.get_json(safeArg.CookieJsonGet.name)
            };
        }

        if ('CookieJsonSet' in safeArg) {
            this.cookie.set_json(safeArg.CookieJsonSet.name, safeArg.CookieJsonSet.value, safeArg.CookieJsonSet.expires_in);
            return null;
        }

        console.info('exec_command: Arg', safeArg);
        return assertNever(safeArg);
    }
}
