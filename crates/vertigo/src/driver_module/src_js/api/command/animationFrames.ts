import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";

/// The frames the browser draws, for as long as wasm wants them: one call into wasm per frame,
/// for all of the app's callbacks at once.
export class AnimationFrames {
    private readonly getWasm: () => ModuleControllerType<ExportType>;
    /// The frame asked for, while the frames go on.
    private request: number | null = null;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    start = () => {
        if (this.request === null) {
            this.request = requestAnimationFrame(this.frame);
        }
    }

    stop = () => {
        if (this.request !== null) {
            cancelAnimationFrame(this.request);
            this.request = null;
        }
    }

    private frame = (time: number) => {
        // Asked for before wasm runs, so that a callback stopping the frames cancels it.
        this.request = requestAnimationFrame(this.frame);

        this.getWasm().wasmCommand({
            AnimationFrame: {
                time,
            },
        });
    }
}
