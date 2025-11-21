import { ModuleControllerType } from "./wasm_init";
import { ExportType } from "./wasm_module";
import { DriverDom } from "./api_browser/dom/dom";
import { AppLocation } from "./exec_command/location/AppLocation";

/**
 * @deprecated
 */
export class ApiBrowser {
    public readonly dom: DriverDom;

    constructor(getWasm: () => ModuleControllerType<ExportType>, appLocation: AppLocation) {
        this.dom = new DriverDom(appLocation, getWasm);
    }
}
