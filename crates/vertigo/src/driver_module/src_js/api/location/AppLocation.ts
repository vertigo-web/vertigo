import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { BrowserLocation } from "./browserLocation";
import { LocationCommonType } from "./types";

type LocationTarget = 'Hash' | 'History';

export class AppLocation {
    private readonly locations: Record<LocationTarget, LocationCommonType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.locations = {
            Hash: new BrowserLocation(
                getWasm,
                "hashchange",
                () => decodeURIComponent(location.hash.substr(1)),
                (value, trigger) => { location.hash = value; trigger(); },
                // No trigger: replacing the hash deliberately does not re-announce.
                (value) => { history.replaceState(null, '', `#${value}`); },
            ),
            History: new BrowserLocation(
                getWasm,
                "popstate",
                () => window.location.pathname + window.location.search + window.location.hash,
                (value, trigger) => { window.history.pushState(null, '', value); trigger(); },
                (value, trigger) => { window.history.replaceState(null, '', value); trigger(); },
            ),
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