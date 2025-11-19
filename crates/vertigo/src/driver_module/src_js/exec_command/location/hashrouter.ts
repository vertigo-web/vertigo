import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { LocationCommonType } from "./types";

export class HashRouter implements LocationCommonType {
    private getWasm: () => ModuleControllerType<ExportType>;
    private callback: Map<CallbackId, () => void>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.callback = new Map();

        window.addEventListener("hashchange", this.trigger);
    }

    private trigger = () => {
        for (const callback of Array.from(this.callback.values())) {
            callback();
        }
    }

    public add = (callback_id: CallbackId) => {
        this.callback.set(callback_id, () => {
            this.getWasm().wasm_command({
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

    public push = (new_hash: string) => {
        if (this.get() === new_hash) {
            return;
        }

        location.hash = new_hash;
        this.trigger();
    }

    public replace = (new_hash: string) => {
        if (this.get() === new_hash) {
            return;
        }

        history.replaceState(null, '', `#${new_hash}`);
    }

    public get(): string {
        return decodeURIComponent(location.hash.substr(1));
    }
}
