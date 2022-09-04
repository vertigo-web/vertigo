import { ModuleControllerType } from "./wasm_init";
import { ExportType } from "./wasm_module";
import { Cookies } from "./api_browser/cookies";
import { Interval } from "./api_browser/interval";
import { HashRouter } from "./api_browser/hashrouter";
import { Fetch } from "./api_browser/fetch";
import { DriverWebsocket } from "./api_browser/websocket/websocket";
import { DriverDom } from "./api_browser/dom/dom";

export class ApiBrowser {
    public readonly cookie: Cookies;
    public readonly interval: Interval;
    public readonly hashRouter: HashRouter;
    public readonly fetch: Fetch;
    public readonly websocket: DriverWebsocket
    public readonly dom: DriverDom;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.cookie = new Cookies();
        this.interval = new Interval(getWasm);
        this.hashRouter = new HashRouter(getWasm);
        this.fetch = new Fetch(getWasm);
        this.websocket = new DriverWebsocket(getWasm);
        this.dom = new DriverDom(getWasm);
    }
}
