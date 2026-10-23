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

/// What the handler in wasm asked of the browser: the flags of a `ClickEvent`, a `WheelEvent` or
/// a `PointerEvent`.
const applyFlags = (event: Event, flags: JsJsonType) => {
    // Check if flags is an object (JsJson Object type)
    if (flags !== null && typeof flags === 'object' && !Array.isArray(flags)) {
        if ('stop_propagation' in flags && flags['stop_propagation'] === true) {
            event.stopPropagation();
        }
        if ('prevent_default' in flags && flags['prevent_default'] === true) {
            event.preventDefault();
        }
        // Only a `PointerEvent` asks for it.
        if ('capture_pointer' in flags && flags['capture_pointer'] === true) {
            try {
                (event.currentTarget as Element).setPointerCapture((event as PointerEvent).pointerId);
            } catch (error) {
                // A pointer that's up already, f. ex. a finger in `pointerup`.
                console.warn('capture_pointer', error);
            }
        }
    }
};

/// Wasm gets the modifier keys and replies with the `ClickEvent` flags. The browser's default
/// action - following a link, submitting a form, toggling a checkbox - goes ahead unless the
/// handler asked to prevent it.
export const click: Handler = (event, send) => {
    const mouseEvent = event as MouseEvent;

    // Payload order MUST match the Rust decoder `ClickEvent::from_driver`.
    applyFlags(event, send([
        mouseEvent.altKey,
        mouseEvent.ctrlKey,
        mouseEvent.shiftKey,
        mouseEvent.metaKey,
    ]));
};

/// Wasm replies with the `WheelEvent` flags: the page scrolls, or zooms, unless the handler
/// asked to prevent it. The pointer goes relative to the element with the listener, the one
/// whose size comes along, rather than to the child under the pointer as `offsetX` would.
export const wheel: Handler = (event, send) => {
    const wheelEvent = event as WheelEvent;
    const rect = (wheelEvent.currentTarget as Element).getBoundingClientRect();

    // Payload order MUST match the Rust decoder `WheelEvent::from_driver`.
    applyFlags(event, send([
        wheelEvent.deltaX,
        wheelEvent.deltaY,
        wheelEvent.deltaZ,
        wheelEvent.deltaMode,
        wheelEvent.clientX - rect.left,
        wheelEvent.clientY - rect.top,
        rect.width,
        rect.height,
        wheelEvent.altKey,
        wheelEvent.ctrlKey,
        wheelEvent.shiftKey,
        wheelEvent.metaKey,
    ]));
};

/// The pointer events, all four of them. Wasm replies with the `PointerEvent` flags, and with
/// `capture_pointer` the events of this pointer go to the element with the listener until it's
/// released. The pointer goes relative to that element, as for the wheel.
export const pointer: Handler = (event, send) => {
    const pointerEvent = event as PointerEvent;
    const rect = (pointerEvent.currentTarget as Element).getBoundingClientRect();

    // Payload order MUST match the Rust decoder `PointerEvent::from_driver`.
    applyFlags(event, send([
        pointerEvent.pointerId,
        pointerEvent.pointerType,
        pointerEvent.isPrimary,
        pointerEvent.button,
        pointerEvent.buttons,
        pointerEvent.clientX - rect.left,
        pointerEvent.clientY - rect.top,
        rect.width,
        rect.height,
        pointerEvent.altKey,
        pointerEvent.ctrlKey,
        pointerEvent.shiftKey,
        pointerEvent.metaKey,
    ]));
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

/// `change` of a checkbox or a radio button, with whether it is checked now - its `value` is
/// the same either way.
const changeChecked: Handler = (event, send) => {
    const target = event.target;

    if (target instanceof HTMLInputElement) {
        send(target.checked);
        return;
    }

    console.warn('changeChecked: not an input', target);
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
/// exactly once here. `hook_keydown`, `change_file` and `change_checked` are vertigo-internal
/// names, remapped to real DOM events where the listener is attached.
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
    change_checked: changeChecked,
    wheel,
    pointerdown: pointer,
    pointermove: pointer,
    pointerup: pointer,
    pointercancel: pointer,
};

/// The DOM event behind a wire name: `change_file` and `change_checked` are the `change` event
/// with a payload of their own.
const domEventName = (event_name: string): string =>
    event_name === 'change_file' || event_name === 'change_checked' ? 'change' : event_name;

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
            // A wheel listener isn't passive, so that `WheelEvent::prevent_default` can keep the
            // page from scrolling: browsers make one on `<body>` passive unless told.
            node.addEventListener(domEventName(event_name), callback, event_name === 'wheel' ? { passive: false } : false);
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
            node.removeEventListener(domEventName(event_name), callback);
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
