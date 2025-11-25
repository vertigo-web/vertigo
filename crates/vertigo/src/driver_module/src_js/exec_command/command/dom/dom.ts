import { ExportType } from "../../../wasm_module";
import { MapNodes } from "./map_nodes";
import { ModuleControllerType } from "../../../wasm_init";
import { JsJsonType } from "../../../jsjson";
import { AppLocation } from "../../location/AppLocation";
import { hydrate } from "./hydration";
import { hydrate_link } from "./injects";
import { getEnableHudration } from "./featureHydration";

interface FileItemType {
    name: string,
    data: Uint8Array,
}

const createElement = (name: string): Element => {
    if (name == "path" || name == "svg") {
        return document.createElementNS("http://www.w3.org/2000/svg", name);
    } else {
        return document.createElement(name);
    }
}

export type CommandType = {
    CreateNode: {
        id: number,
        name: string,
    }
} | {
    CreateText: {
        id: number,
        value: string
    }
} | {
    UpdateText: {
        id: number,
        value: string
    }
} | {
    SetAttr: {
        id: number,
        name: string,
        value: string
    }
} | {
    RemoveAttr: {
        id: number,
        name: string
    }
} | {
    RemoveNode: {
        id: number,
    }
} | {
    RemoveText: {
        id: number,
    }
} | {
    InsertBefore: {
        parent: number,
        child: number,
        ref_id: number | null,
    }
} | {
    InsertCss: {
        selector: string | null,
        value: string
    }
} | {
    CreateComment: {
        id: number,
        value: string
    }
} | {
    RemoveComment: {
        id: number,
    }
} | {
    CallbackAdd: {
        id: number,
        event_name: string,
        callback_id: number,
    }
} | {
    CallbackRemove: {
        id: number,
        event_name: string,
        callback_id: number,
    }
};

const assertNeverCommand = (data: never): never => {
    console.error(data);
    throw Error('unknown command');
};

export class DriverDom {
    private appLocation: AppLocation;
    private readonly getWasm: () => ModuleControllerType<ExportType>;
    public readonly nodes: MapNodes;
    private callbacks: Map<bigint, (data: Event) => void>;

    public constructor(appLocation: AppLocation, getWasm: () => ModuleControllerType<ExportType>) {
        this.appLocation = appLocation;
        this.getWasm = getWasm;
        this.nodes = new MapNodes();
        this.callbacks = new Map();

        document.addEventListener('dragover', (ev): void => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
    }

    public debugNodes(...ids: Array<number>) {
        const result: Record<number, unknown> = {};
        for (const id of ids) {
            const value = this.nodes.get_any_option(id);
            result[id] = value;
        }
        console.info('debug nodes', result);
    }

    private wasm_callback(callback_id: bigint, value: JsJsonType): JsJsonType {
        return this.getWasm().wasm_command({
            CallbackCall: {
                callback_id: Number(callback_id),
                value: value
            }
        });
    }

    private create_node(id: number, name: string) {
        if (this.nodes.has(id)) {
            return;
        }

        const node = createElement(name);
        this.nodes.set(id, node);

        if (name.toLowerCase().trim() === 'a') {
            hydrate_link(node, this.appLocation);
        }
    }

    private set_attribute(id: number, name: string, value: string) {
        const node = this.nodes.get_node("set_attribute", id);
        node.setAttribute(name, value);

        if (name == "value") {
            if (node instanceof HTMLInputElement) {
                node.value = value;
                return;
            }

            if (node instanceof HTMLTextAreaElement) {
                node.value = value;
                node.defaultValue = value;
                return;
            }
        }
    }

    private remove_attribute(id: number, name: string) {
        const node = this.nodes.get_node("remove_attribute", id);
        node.removeAttribute(name);

        if (name == "value") {
            if (node instanceof HTMLInputElement) {
                node.value = "";
                return;
            }

            if (node instanceof HTMLTextAreaElement) {
                node.value = "";
                node.defaultValue = "";
                return;
            }
        }
    }

    private remove_node(id: number) {
        const node = this.nodes.delete("remove_node", id);
        node.remove();
    }

    private create_text(id: number, value: string) {
        if (this.nodes.has(id)) {
            return;
        }

        const text = document.createTextNode(value);
        this.nodes.set(id, text);
    }

    private remove_text(id: number) {
        const text = this.nodes.delete("remove_node", id);
        text.remove();
    }

    private update_text(id: number, value: string) {
        const text = this.nodes.get_text("set_attribute", id);
        text.textContent = value;
    }

    private callback_click(event: Event, callback_id: bigint) {
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

    private callback_submit(event: Event, callback_id: bigint) {
        event.preventDefault();
        this.wasm_callback(callback_id, undefined);
    }

    private callback_input(event: Event, callback_id: bigint) {
        const target = event.target;

        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
            this.wasm_callback(callback_id, target.value);
            return;
        }

        console.warn('event input ignore', target);
    }

    private callback_change(event: Event, callback_id: bigint) {
        const target = event.target;

        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) {
            this.wasm_callback(callback_id, target.value);
            return;
        }

        console.warn('event input ignore', target);
    }

    private callback_blur(_event: Event, callback_id: bigint) {
        this.wasm_callback(callback_id, undefined);
    }

    private callback_mousedown(event: Event, callback_id: bigint) {
        if (this.wasm_callback(callback_id, undefined)) {
            event.preventDefault()
        }
    }

    private callback_mouseup(event: Event, callback_id: bigint) {
        if (this.wasm_callback(callback_id, undefined)) {
            event.preventDefault()
        }
    }

    private callback_mouseenter(_event: Event, callback_id: bigint) {
        this.wasm_callback(callback_id, undefined);
    }

    private callback_mouseleave(_event: Event, callback_id: bigint) {
        this.wasm_callback(callback_id, undefined);
    }

    private callback_drop(event: Event, callback_id: bigint) {
        event.preventDefault();

        if (event instanceof DragEvent) {
            if (event.dataTransfer === null) {
                console.error('dom -> drop -> dataTransfer null');
            } else {
                const files: Array<Promise<FileItemType>> = [];

                for (let i = 0; i < event.dataTransfer.items.length; i++) {
                    const item = event.dataTransfer.items[i];

                    if (item === undefined) {
                        console.error('dom -> drop -> item - undefined');
                    } else {
                        const file = item.getAsFile();

                        if (file === null) {
                            console.error(`dom -> drop -> index:${i} -> It's not a file`);
                        } else {
                            files.push(file
                                .arrayBuffer()
                                .then((data): FileItemType => ({
                                    name: file.name,
                                    data: new Uint8Array(data),
                                }))
                            );
                        }
                    }
                }

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

    private callback_keydown(event: Event, callback_id: bigint) {
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

    private callback_load(event: Event, callback_id: bigint) {
        event.preventDefault();
        this.wasm_callback(callback_id, undefined);
    }

    private callback_add(id: number, event_name: string, callback_id: bigint) {
        const callback = (event: Event) => {
            if (event_name === 'click') {
                return this.callback_click(event, callback_id);
            }

            if (event_name === 'submit') {
                return this.callback_submit(event, callback_id);
            }

            if (event_name === 'input') {
                return this.callback_input(event, callback_id);
            }

            if (event_name === 'change') {
                return this.callback_change(event, callback_id);
            }

            if (event_name === 'blur') {
                return this.callback_blur(event, callback_id);
            }

            if (event_name === 'mousedown') {
                return this.callback_mousedown(event, callback_id);
            }

            if (event_name === 'mouseup') {
                return this.callback_mouseup(event, callback_id);
            }

            if (event_name === 'mouseenter') {
                return this.callback_mouseenter(event, callback_id);
            }

            if (event_name === 'mouseleave') {
                return this.callback_mouseleave(event, callback_id);
            }

            if (event_name === 'keydown') {
                return this.callback_keydown(event, callback_id);
            }

            if (event_name === 'hook_keydown') {
                return this.callback_keydown(event, callback_id);
            }

            if (event_name === 'drop') {
                return this.callback_drop(event, callback_id);
            }

            if (event_name === 'load') {
                return this.callback_load(event, callback_id);
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
            const node = this.nodes.get('callback_add', id);
            node.addEventListener(event_name, callback, false);
        }
    }

    private callback_remove(id: number, event_name: string, callback_id: bigint) {
        const callback = this.callbacks.get(callback_id);
        this.callbacks.delete(callback_id);

        if (callback === undefined) {
            console.error(`The callback is missing with the id=${callback_id}`);
            return;
        }

        if (event_name === 'hook_keydown') {
            document.removeEventListener('keydown', callback);
        } else {
            const node = this.nodes.get('callback_remove', id);
            node.removeEventListener(event_name, callback);
        }
    }

    public dom_bulk_update = (commands: Array<CommandType>) => {
        if (getEnableHudration()) {
            hydrate(commands, this.appLocation);
        }

        const setFocus: Set<number> = new Set();

        for (const command of commands) {
            try {
                this.bulk_update_command(command);
            } catch (error) {
                console.error('bulk_update - item', error, command);
            }

            if ('SetAttr' in command && command.SetAttr.name.toLocaleLowerCase() === 'autofocus') {
                setFocus.add(command.SetAttr.id);
            }
        }

        if (setFocus.size > 0) {
            setTimeout(() => {
                for (const id of setFocus) {
                    const node = this.nodes.get_node_element(`set focus ${id}`, id);
                    node.focus();
                }
            }, 0);
        }

        if (getEnableHudration() === false) {
            this.nodes.removeInitNodes();
        }
    }

    private bulk_update_command(command: CommandType) {
        if ('RemoveNode' in command) {
            this.remove_node(command.RemoveNode.id);
            return;
        }

        if ('InsertBefore' in command) {
            this.nodes.insert_before(command.InsertBefore.parent, command.InsertBefore.child, command.InsertBefore.ref_id === null ? null : command.InsertBefore.ref_id);
            return;
        }

        if ('CreateNode' in command) {
            this.create_node(command.CreateNode.id, command.CreateNode.name);
            return;
        }

        if ('CreateText' in command) {
            this.create_text(command.CreateText.id, command.CreateText.value);
            return;
        }

        if ('UpdateText' in command) {
            this.update_text(command.UpdateText.id, command.UpdateText.value);
            return;
        }

        if ('SetAttr' in command) {
            this.set_attribute(command.SetAttr.id, command.SetAttr.name, command.SetAttr.value);
            return;
        }

        if ('RemoveAttr' in command) {
            this.remove_attribute(command.RemoveAttr.id, command.RemoveAttr.name);
            return;
        }

        if ('RemoveText' in command) {
            this.remove_text(command.RemoveText.id);
            return;
        }

        if ('InsertCss' in command) {
            this.nodes.insert_css(command.InsertCss.selector, command.InsertCss.value);
            return;
        }

        if ('CreateComment' in command) {
            const comment = document.createComment(command.CreateComment.value);
            this.nodes.set(command.CreateComment.id, comment);
            return;
        }

        if ('RemoveComment' in command) {
            const comment = this.nodes.delete("remove_comment", command.RemoveComment.id);
            comment.remove();
            return;
        }

        if ('CallbackAdd' in command) {
            this.callback_add(command.CallbackAdd.id, command.CallbackAdd.event_name, BigInt(command.CallbackAdd.callback_id));
            return;
        }

        if ('CallbackRemove' in command) {
            this.callback_remove(command.CallbackRemove.id, command.CallbackRemove.event_name, BigInt(command.CallbackRemove.callback_id));
            return;
        }

        return assertNeverCommand(command);
    }
}
