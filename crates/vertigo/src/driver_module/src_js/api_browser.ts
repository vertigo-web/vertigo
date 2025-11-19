import { ModuleControllerType } from "./wasm_init";
import { ExportType } from "./wasm_module";
import { Cookies } from "./api_browser/cookies";
import { HashRouter } from "./exec_command/location/hashrouter";
import { DriverDom } from "./api_browser/dom/dom";
import { AppLocation } from "./exec_command/location/AppLocation";

/**
 * @deprecated
 */
export class ApiBrowser {
    public readonly cookie: Cookies;
    public readonly hashRouter: HashRouter;
    public readonly dom: DriverDom;

    constructor(getWasm: () => ModuleControllerType<ExportType>, appLocation: AppLocation) {
        this.cookie = new Cookies();
        this.hashRouter = new HashRouter(getWasm);
        this.dom = new DriverDom(appLocation, getWasm);
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
