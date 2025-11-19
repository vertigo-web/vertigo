import { ModuleControllerType } from "./wasm_init";
import { ExportType } from "./wasm_module";
import { Cookies } from "./api_browser/cookies";
import { HashRouter } from "./api_browser/hashrouter";
import { DriverDom } from "./api_browser/dom/dom";
import { HistoryLocation } from "./api_browser/historyLocation";

/**
 * @deprecated
 */
export class ApiBrowser {
    public readonly cookie: Cookies;
    public readonly hashRouter: HashRouter;
    public readonly historyLocation: HistoryLocation;
    public readonly dom: DriverDom;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.cookie = new Cookies();
        this.hashRouter = new HashRouter(getWasm);
        this.historyLocation = new HistoryLocation(getWasm);
        this.dom = new DriverDom(this.historyLocation, getWasm);
    }

    public getRandom = (min: number, max: number): number => {
        const range = max - min + 1;
        let result = Math.floor(Math.random() * range);
        return min + result;
    }

    public get_env = (name: string): string | null => {
        return document.documentElement.getAttribute(`data-env-${name}`);
    }

    public getTimezoneOffset = (): number => {
        return new Date().getTimezoneOffset()
    }
}
