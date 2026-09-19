import { ExportType } from "../../../wasm_module";
import { getFiles, readFile, sendFiles, Send } from "./dataTransfer";
import { JsJsonType } from "../../../jsjson";
import { ModuleControllerType } from "../../../wasm_init";
import { MapNodes } from "./map_nodes";
import { CallbackId } from "../../types";

/// One DOM event, turned into at most one call into wasm.
///
/// `send` is bound to the callback id at registration, so a handler never needs to know it.
type Handler = (event: Event, send: Send) => void;

const notify: Handler = (_event, send) => {
    send(undefined);
};

const notifyPreventDefault: Handler = (event, send) => {
    event.preventDefault();
    send(undefined);
};

/// Wasm decides: a truthy reply means the default was not wanted.
const notifyPreventIfTrue: Handler = (event, send) => {
    if (send(undefined)) {
        event.preventDefault();
    }
};

const click: Handler = (event, send) => {
    event.preventDefault();
    let click_event = send(undefined);

    // Check if click_event is an object (JsJson Object type)
    if (click_event !== null && typeof click_event === 'object' && !Array.isArray(click_event)) {
        if ('stop_propagation' in click_event && click_event['stop_propagation'] === true) {
            event.stopPropagation();
        }
        if ('prevent_default' in click_event && click_event['prevent_default'] === true) {
            event.preventDefault();
        }
    }
};

/// `input` and `change` differ only in whether a `<select>` counts as a value source. They
/// also share a warning, which stays one literal rather than two identical copies.
const readValue = (allowSelect: boolean): Handler => (event, send) => {
    const target = event.target;

    if (target instanceof HTMLInputElement
        || target instanceof HTMLTextAreaElement
        || (allowSelect && target instanceof HTMLSelectElement)) {
        send(target.value);
        return;
    }

    console.warn('event input ignore', target);
};

const changeFile: Handler = (event, send) => {
    const target = event.target;

    if (target instanceof HTMLInputElement && target.files !== null && target.files.length > 0) {
        const promises = [];

        for (let i = 0; i < target.files.length; i++) {
            const file = target.files[i];
            if (file !== undefined) {
                promises.push(readFile(file));
            }
        }

        if (promises.length > 0) {
            sendFiles(promises, send, 'changeFile ->');
        }

        target.value = '';
        return;
    }

    console.warn('changeFile: not a file input or no files', target);
};

const drop: Handler = (event, send) => {
    event.preventDefault();

    if (event instanceof DragEvent) {
        if (event.dataTransfer === null) {
            console.error('dom -> drop -> dataTransfer null');
        } else {
            const files = getFiles(event.dataTransfer.items);

            if (files.length) {
                sendFiles(files, send, 'callback_drop -> promise.all -> ');
            } else {
                console.error('No files to send');
            }
        }
    } else {
        console.warn('event drop ignore', event);
    }
};

const keydown: Handler = (event, send) => {
    if (event instanceof KeyboardEvent) {
        const result = send([
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
};

/// The event names are wire strings from Rust (`dom/dom_element.rs`), so each is spelled out
/// exactly once here. `hook_keydown` and `change_file` are vertigo-internal names, remapped
/// to real DOM events where the listener is attached.
const HANDLERS: Record<string, Handler> = {
    click,
    submit: notifyPreventDefault,
    input: readValue(false),
    change: readValue(true),
    blur: notify,
    mousedown: notifyPreventIfTrue,
    mouseup: notifyPreventIfTrue,
    mouseenter: notify,
    mouseleave: notify,
    keydown,
    hook_keydown: keydown,
    drop,
    load: notifyPreventDefault,
    change_file: changeFile,
};

export class CallbackManager {
    private readonly getWasm: () => ModuleControllerType<ExportType>;
    private callbacks: Map<CallbackId, (data: Event) => void>;
    // IntersectionObserver does not use addEventListener, so its observers are
    // tracked separately (keyed by callback_id) for disconnect on remove.
    private observers: Map<CallbackId, IntersectionObserver>;

    public constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
        this.callbacks = new Map();
        this.observers = new Map();
    }

    public add(nodes: MapNodes, id: number, event_name: string, callback_id: CallbackId) {
        if (event_name === 'intersect') {
            return this.intersectAdd(nodes, id, callback_id);
        }

        const send: Send = (value) => this.wasmCallback(callback_id, value);

        const callback = (event: Event) => {
            // Looked up per fire rather than once at registration, which keeps the previous
            // behaviour for an unsupported name: the listener is still attached and the
            // complaint is logged each time the event arrives.
            const handler = HANDLERS[event_name];

            if (handler === undefined) {
                console.error(`No support for the event ${event_name}`);
                return;
            }

            handler(event, send);
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
            const domEventName = event_name === 'change_file' ? 'change' : event_name;
            node.addEventListener(domEventName, callback, false);
        }
    }

    public remove(nodes: MapNodes, id: number, event_name: string, callback_id: CallbackId) {
        if (event_name === 'intersect') {
            return this.intersectRemove(callback_id);
        }

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
            const domEventName = event_name === 'change_file' ? 'change' : event_name;
            node.removeEventListener(domEventName, callback);
        }
    }

    private wasmCallback(callback_id: CallbackId, value: JsJsonType): JsJsonType {
        return this.getWasm().wasmCommand({
            CallbackCall: {
                callback_id,
                value: value
            }
        });
    }

    private intersectAdd(nodes: MapNodes, id: number, callback_id: CallbackId) {
        if (this.observers.has(callback_id)) {
            console.error(`There was already an intersect observer added with the callback_id=${callback_id}`);
            return;
        }

        const node = nodes.getNode('callback_add', id);

        const observer = new IntersectionObserver((entries) => {
            for (const entry of entries) {
                // Payload order MUST match the Rust decoder get_intersection_event.
                this.wasmCallback(callback_id, [
                    entry.isIntersecting,
                    entry.intersectionRatio,
                    entry.boundingClientRect.top,
                    entry.boundingClientRect.bottom,
                    entry.boundingClientRect.height,
                ]);
            }
        });

        observer.observe(node);
        this.observers.set(callback_id, observer);
    }

    private intersectRemove(callback_id: CallbackId) {
        const observer = this.observers.get(callback_id);
        this.observers.delete(callback_id);

        if (observer === undefined) {
            console.error(`The intersect observer is missing with the id=${callback_id}`);
            return;
        }

        observer.disconnect();
    }
}
