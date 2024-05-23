import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";
export class HistoryLocation {
    private getWasm: () => ModuleControllerType<ExportType>;
    private callback: Map<bigint, () => void>;

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

    public add = (callback_id: bigint) => {
        this.callback.set(callback_id, () => {
            this.getWasm().wasm_callback(callback_id, this.get());
        });
    }

    public remove = (callback_id: bigint) => {
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
