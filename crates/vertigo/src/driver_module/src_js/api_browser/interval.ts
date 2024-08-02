import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class Interval {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public interval_set = (duration: number, callback_id: bigint): ReturnType<typeof setTimeout> => {
        const timer_id = setInterval(() => {
            this.getWasm().wasm_callback(callback_id, undefined);
        }, duration);

        return timer_id;
    }

    public interval_clear = (timer_id: number) => {
        clearInterval(timer_id);
    }

    timeout_set = (duration: number, callback_id: bigint): ReturnType<typeof setTimeout> => {
        const timeout_id = setTimeout(() => {
            this.getWasm().wasm_callback(callback_id, undefined);
        }, duration);

        return timeout_id;
    }

    timeout_clear = (timer_id: number): void => {
        clearTimeout(timer_id);
    }
}
