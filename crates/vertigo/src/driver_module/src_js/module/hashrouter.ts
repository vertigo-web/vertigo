import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class HashRouter {
    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        window.addEventListener("hashchange", () => {
            const params = getWasm().newList();
            params.push_string(this.get());
            const listId = params.freeze();

            getWasm().exports.hashrouter_hashchange_callback(listId);
        }, false);
    }

    public push = (new_hash: string) => {
        location.hash = new_hash;
    }

    public get(): string {
        return location.hash.substr(1);
    }
}
