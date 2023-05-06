import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class HashRouter {
    private getWasm: () => ModuleControllerType<ExportType>;
    private callback: Map<bigint, () => void>;

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

    public add = (callback_id: bigint) => {
        this.callback.set(callback_id, () => {
            this.getWasm().wasm_callback(callback_id, this.get());
        });
    }

    public remove = (callback_id: bigint) => {
        this.callback.delete(callback_id);
    }

    public push = (new_hash: string) => {
        if (this.get() === new_hash) {
            return;
        }

        location.hash = new_hash;
        this.trigger();
    }

    public get(): string {
        return decodeURIComponent(location.hash.substr(1));
    }
}
