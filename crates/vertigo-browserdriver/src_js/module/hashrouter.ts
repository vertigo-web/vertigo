import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class HashRouter {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;

        window.addEventListener("hashchange", () => {
            this.hashrouter_get_hash_location();
            this.getWasm().exports.hashrouter_hashchange_callback();
        }, false);
    }

    public hashrouter_get_hash_location = () => {                     //zwraca przez stos stringa
        const currentHash = location.hash.substr(1);
        this.getWasm().pushString(currentHash);
    }

    public hashrouter_push_hash_location = (/*new_hash: string*/ new_hash_ptr: BigInt, new_hash_length: BigInt) => {
        location.hash = this.getWasm().decodeText(new_hash_ptr, new_hash_length);
    }
}
