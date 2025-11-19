import { DriverWebsocket } from "./websocket/websocket";
import { assertNever } from "../assert_never";
import { JsJsonType } from "../jsjson";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";
import { fetchCacheGet } from "./command/fetchCacheGet";
import { fetchExec, FetchRequestType } from "./command/fetchExec";
import { CallbackId } from "./types";
import { Interval } from "../api_browser/interval";

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
    };



export class ExecCommand {
    private readonly websocket: DriverWebsocket;
    private readonly interval: Interval;
    

    constructor(private readonly getWasm: () => ModuleControllerType<ExportType>) {
        this.websocket = new DriverWebsocket(getWasm);
        this.interval = new Interval(getWasm);
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
            this.interval.TimerClear(safeArg.TimerClear.callback);
            return null;
        }

        console.info('exec_command: Arg', safeArg);
        return assertNever(safeArg);
    }
}
