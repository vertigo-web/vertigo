import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class HashRouter {
    private getWasm: () => ModuleControllerType<ExportType>;
    private callback: Map<bigint, () => void>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.callback = new Map();
    }

    public add = (callback_id: bigint) => {
        const callback = () => {
            this.getWasm().wasm_callback(callback_id, this.get());
        };

        window.addEventListener("hashchange", callback);
        this.callback.set(callback_id, callback);
    }

    public remove = (callback_id: bigint) => {
        const callback = this.callback.get(callback_id);
        if (callback === undefined) {
            console.error(`HashRouter - The callback with id is missing = ${callback_id}`);
            return;
        }

        this.callback.delete(callback_id);
        window.removeEventListener('hashchange', callback);
    }

    public push = (new_hash: string) => {
        location.hash = new_hash;
    }

    public get(): string {
        return decodeURIComponent(location.hash.substr(1));
    }
}
