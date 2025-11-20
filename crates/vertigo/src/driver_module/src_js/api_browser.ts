import { ModuleControllerType } from "./wasm_init";
import { ExportType } from "./wasm_module";
import { HashRouter } from "./exec_command/location/hashrouter";
import { DriverDom } from "./api_browser/dom/dom";
import { AppLocation } from "./exec_command/location/AppLocation";

/**
 * @deprecated
 */
export class ApiBrowser {
    public readonly hashRouter: HashRouter;
    public readonly dom: DriverDom;

    constructor(getWasm: () => ModuleControllerType<ExportType>, appLocation: AppLocation) {
        this.hashRouter = new HashRouter(getWasm);
        this.dom = new DriverDom(appLocation, getWasm);
    }

    public getRandom = (min: number, max: number): number => {
        const range = max - min + 1;
        let result = Math.floor(Math.random() * range);
        return min + result;
    }

    public getTimezoneOffset = (): number => {
        return new Date().getTimezoneOffset()
    }
}
