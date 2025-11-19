import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { HashRouter } from "./hashrouter";
import { HistoryLocation } from "./historyLocation";
import { LocationCommonType } from "./types";

type LocationTarget = 'Hash' | 'History';

export class AppLocation {
    private readonly locations: Record<LocationTarget, LocationCommonType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.locations = {
            Hash: new HashRouter(getWasm),
            History: new HistoryLocation(getWasm),
        };
    }
    
    callback = (target: LocationTarget, mode: 'Add' | 'Remove', callbackId: CallbackId) => {
        switch (mode) {
            case 'Add': {
                this.locations[target].add(callbackId);
                return;
            }
            case 'Remove': {
                this.locations[target].remove(callbackId);
                return;
            }
        }
    }

    set = (target: LocationTarget, mode: 'Push' | 'Replace', newValue: string) => {
        switch (mode) {
            case 'Push': {
                this.locations[target].push(newValue);
                return;
            }
            case 'Replace': {
                 this.locations[target].replace(newValue);
                 return;
            }
        }
    }

    get = (target: LocationTarget): string => {
        return this.locations[target].get();
    }
}