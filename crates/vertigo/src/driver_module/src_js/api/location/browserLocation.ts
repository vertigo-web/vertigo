import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";
import { LocationCommonType } from "./types";

/// The hash router and the history router, which differ only in the event they listen for,
/// how the current location is read, and what the two setters do.
///
/// The setters take `trigger` rather than being wrapped by it, because the two are not
/// symmetric: both routers announce a `push`, but only the history router announces a
/// `replace` - changing the hash via `replaceState` deliberately stays quiet.
export class BrowserLocation implements LocationCommonType {
    private callback: Map<CallbackId, () => void> = new Map();

    constructor(
        private readonly getWasm: () => ModuleControllerType<ExportType>,
        event: 'hashchange' | 'popstate',
        /// A parameter property, so this *is* `LocationCommonType.get` - no forwarding method.
        public readonly get: () => string,
        private readonly pushValue: (value: string, trigger: () => void) => void,
        private readonly replaceValue: (value: string, trigger: () => void) => void,
    ) {
        window.addEventListener(event, this.trigger);
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

    public push = (value: string) => {
        if (this.get() === value) {
            return;
        }

        this.pushValue(value, this.trigger);
    }

    public replace = (value: string) => {
        if (this.get() === value) {
            return;
        }

        this.replaceValue(value, this.trigger);
    }
}
