import { CallbackId } from "../exec_command/types";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

type TimerResourceId = ReturnType<typeof setTimeout>;

interface TimerId {
    kind: 'Interval' | 'Timeout',
    timerId: TimerResourceId,
}

export class Interval {
    private readonly getWasm: () => ModuleControllerType<ExportType>;
    private readonly data: Map<CallbackId, TimerId>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.data = new Map();
    }

    timerSet = (callback: CallbackId, duration: number, kind: 'Interval' | 'Timeout') => {
        switch (kind) {
            case 'Interval': {
                const timerId = setInterval(() => {
                    this.getWasm().wasm_command({
                        'TimerCall': {
                            callback,
                        },
                    })
                }, duration);

                this.data.set(callback, {
                    kind: 'Interval',
                    timerId,
                });
                break;
            }
            case 'Timeout': {
                const timerId = setTimeout(() => {
                    this.getWasm().wasm_command({
                        'TimerCall': {
                            callback,
                        },
                    })
                }, duration);

                this.data.set(callback, {
                    kind: 'Timeout',
                    timerId,
                });
                break;
            }
        }
    }

    TimerClear = (callback: CallbackId) => {
        const timerResource = this.data.get(callback);

        if (timerResource === undefined) {
            throw Error('panic');
        }

        switch (timerResource.kind) {
            case 'Interval': {
                clearInterval(timerResource.timerId);
                break;
            }
            case 'Timeout': {
                clearTimeout(timerResource.timerId);
                break;
            }
        }
    }
}
