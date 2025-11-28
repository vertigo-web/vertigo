import { ExportType } from "../../../wasm_module";
import { getFiles } from "./dataTransfer";
import { JsJsonType } from "../../../jsjson";
import { ModuleControllerType } from "../../../wasm_init";
import { MapNodes } from "./map_nodes";

export class CallbackManager {
    private readonly getWasm: () => ModuleControllerType<ExportType>;
    private callbacks: Map<bigint, (data: Event) => void>;

    public constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.callbacks = new Map();
    }

    public add(nodes: MapNodes, id: number, event_name: string, callback_id: bigint) {
        const callback = (event: Event) => {
            if (event_name === 'click') {
                return this.click(event, callback_id);
            }

            if (event_name === 'submit') {
                return this.submit(event, callback_id);
            }

            if (event_name === 'input') {
                return this.input(event, callback_id);
            }

            if (event_name === 'change') {
                return this.change(event, callback_id);
            }

            if (event_name === 'blur') {
                return this.blur(event, callback_id);
            }

            if (event_name === 'mousedown') {
                return this.mousedown(event, callback_id);
            }

            if (event_name === 'mouseup') {
                return this.mouseup(event, callback_id);
            }

            if (event_name === 'mouseenter') {
                return this.mouseenter(event, callback_id);
            }

            if (event_name === 'mouseleave') {
                return this.mouseleave(event, callback_id);
            }

            if (event_name === 'keydown') {
                return this.keydown(event, callback_id);
            }

            if (event_name === 'hook_keydown') {
                return this.keydown(event, callback_id);
            }

            if (event_name === 'drop') {
                return this.drop(event, callback_id);
            }

            if (event_name === 'load') {
                return this.load(event, callback_id);
            }

            console.error(`No support for the event ${event_name}`);
        };

        if (this.callbacks.has(callback_id)) {
            console.error(`There was already a callback added with the callback_id=${callback_id}`);
            return;
        }

        this.callbacks.set(callback_id, callback);

        if (event_name === 'hook_keydown') {
            document.addEventListener('keydown', callback, false);
        } else {
            const node = nodes.get('callback_add', id);
            node.addEventListener(event_name, callback, false);
        }
    }

    public remove(nodes: MapNodes, id: number, event_name: string, callback_id: bigint) {
        const callback = this.callbacks.get(callback_id);
        this.callbacks.delete(callback_id);

        if (callback === undefined) {
            console.error(`The callback is missing with the id=${callback_id}`);
            return;
        }

        if (event_name === 'hook_keydown') {
            document.removeEventListener('keydown', callback);
        } else {
            const node = nodes.get('callback_remove', id);
            node.removeEventListener(event_name, callback);
        }
    }

    private wasm_callback(callback_id: bigint, value: JsJsonType): JsJsonType {
        return this.getWasm().wasm_command({
            CallbackCall: {
                callback_id: Number(callback_id),
                value: value
            }
        });
    }

    private click(event: Event, callback_id: bigint) {
        event.preventDefault();
        let click_event = this.wasm_callback(callback_id, undefined);

        // Check if click_event is an object (JsJson Object type)
        if (click_event !== null && typeof click_event === 'object' && !Array.isArray(click_event)) {
            if ('stop_propagation' in click_event && click_event['stop_propagation'] === true) {
                event.stopPropagation();
            }
            if ('prevent_default' in click_event && click_event['prevent_default'] === true) {
                event.preventDefault();
            }
        }
    }

    private submit(event: Event, callback_id: bigint) {
        event.preventDefault();
        this.wasm_callback(callback_id, undefined);
    }

    private input(event: Event, callback_id: bigint) {
        const target = event.target;

        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
            this.wasm_callback(callback_id, target.value);
            return;
        }

        console.warn('event input ignore', target);
    }

    private change(event: Event, callback_id: bigint) {
        const target = event.target;

        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) {
            this.wasm_callback(callback_id, target.value);
            return;
        }

        console.warn('event input ignore', target);
    }

    private blur(_event: Event, callback_id: bigint) {
        this.wasm_callback(callback_id, undefined);
    }

    private mousedown(event: Event, callback_id: bigint) {
        if (this.wasm_callback(callback_id, undefined)) {
            event.preventDefault()
        }
    }

    private mouseup(event: Event, callback_id: bigint) {
        if (this.wasm_callback(callback_id, undefined)) {
            event.preventDefault()
        }
    }

    private mouseenter(_event: Event, callback_id: bigint) {
        this.wasm_callback(callback_id, undefined);
    }

    private mouseleave(_event: Event, callback_id: bigint) {
        this.wasm_callback(callback_id, undefined);
    }

    private drop(event: Event, callback_id: bigint) {
        event.preventDefault();

        if (event instanceof DragEvent) {
            if (event.dataTransfer === null) {
                console.error('dom -> drop -> dataTransfer null');
            } else {
                const files = getFiles(event.dataTransfer.items);

                if (files.length) {
                    Promise.all(files).then((files) => {
                        const params = [];

                        for (const file of files) {
                            // Convert Uint8Array to array of numbers for JsJson
                            const dataArray = Array.from(file.data);
                            params.push([
                                file.name,
                                dataArray,
                            ]);
                        }

                        this.wasm_callback(callback_id, [params]);
                    }).catch((error) => {
                        console.error('callback_drop -> promise.all -> ', error);
                    });
                } else {
                    console.error('No files to send');
                }
            }
        } else {
            console.warn('event drop ignore', event);
        }
    }

    private keydown(event: Event, callback_id: bigint) {
        if (event instanceof KeyboardEvent) {
            const result = this.wasm_callback(callback_id, [
                event.key,
                event.code,
                event.altKey,
                event.ctrlKey,
                event.shiftKey,
                event.metaKey
            ]);

            if (result === true) {
                event.preventDefault();
                event.stopPropagation();
            }

            return;
        }

        console.warn('keydown ignore', event);
    }

    private load(event: Event, callback_id: bigint) {
        event.preventDefault();
        this.wasm_callback(callback_id, undefined);
    }

}
