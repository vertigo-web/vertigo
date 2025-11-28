import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { LocationCommonType } from "./types";

export class HistoryLocation implements LocationCommonType {
    private getWasm: () => ModuleControllerType<ExportType>;
    private callback: Map<CallbackId, () => void>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.callback = new Map();

        window.addEventListener("popstate", this.trigger);
    }

    private trigger = () => {
        for (const callback of Array.from(this.callback.values())) {
            callback();
        }
    }

    public add = (callback_id: CallbackId) => {
        this.callback.set(callback_id, () => {
            this.getWasm().wasmCommand({
                LocationCall: {
                    callback: callback_id,
                    value: this.get(),
                }
            });
        });
    }

    public remove = (callback_id: CallbackId) => {
        this.callback.delete(callback_id);
    }

    public push = (url: string) => {
        if (this.get() === url) {
            return;
        }

        window.history.pushState(null, '', url);
        this.trigger();
    }

    public replace = (url: string) => {
        if (this.get() === url) {
            return;
        }

        window.history.replaceState(null, '', url);
        this.trigger();
    }

    public get(): string {
        return window.location.pathname + window.location.search + window.location.hash;
    }
}
