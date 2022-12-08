import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { HistoryLocation } from "../historyLocation";
import { MapNodes } from "./map_nodes";

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

type CommandType = {
    type: 'create_node',
    id: number,
    name: string,
} | {
    type: 'create_text',
    id: number,
    value: string
} | {
    type: 'update_text',
    id: number,
    value: string
} | {
    type: 'set_attr',
    id: number,
    name: string,
    value: string
} | {
    type: 'remove_node',
    id: number,
} | {
    type: 'remove_text',
    id: number,
} | {
    type: 'insert_before',
    parent: number,
    child: number,
    ref_id: number | null,
} | {
    type: 'insert_css',
    selector: string,
    value: string
} | {
    type: 'create_comment',
    id: number,
    value: string
} | {
    type: 'remove_comment',
    id: number,
} | {
    type: 'callback_add',
    id: number,
    event_name: string,
    callback_id: number,
} | {
    type: 'callback_remove',
    id: number,
    event_name: string,
    callback_id: number,
};

const assertNeverCommand = (data: never): never => {
    console.error(data);
    throw Error('unknown command');
};

export class DriverDom {
    private root: HTMLElement;
    private historyLocation: HistoryLocation;
    private readonly getWasm: () => ModuleControllerType<ExportType>;
    public readonly nodes: MapNodes<number, Element | Comment>;
    public readonly texts: MapNodes<number, Text>;
    private callbacks: Map<bigint, (data: Event) => void>;
    private initBeforeFirstUpdate: boolean;

    public constructor(root: HTMLElement, historyLocation: HistoryLocation, getWasm: () => ModuleControllerType<ExportType>) {
        this.root = root;
        this.historyLocation = historyLocation;
        this.getWasm = getWasm;
        this.nodes = new MapNodes();
        this.texts = new MapNodes();
        this.callbacks = new Map();
        this.initBeforeFirstUpdate = false;

        this.nodes.set(1, root);

        document.addEventListener('dragover', (ev): void => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
    }

    public debugNodes(...ids: Array<number>) {
        const result: Record<number, unknown> = {};
        for (const id of ids) {
            const value = this.nodes.getItem(id);
            result[id] = value;
        }
        console.info('debug nodes', result);
    }

    private create_node(id: number, name: string) {
        if (id === 1) {
            console.error("The root HTMLElement is already created");
        } else {
            const node = createElement(name);
            node.setAttribute('data-id', id.toString());
            this.nodes.set(id, node);

            if (name.toLowerCase().trim() === 'a') {
                node.addEventListener('click', (e) => {
                    let href = node.getAttribute('href');
                    if (href === null) {
                        return;
                    }

                    if (href.startsWith('http://') || href.startsWith('https://') || href.startsWith('//')) {
                        return;
                    }

                    e.preventDefault();
                    this.historyLocation.push(href);
                    window.scrollTo(0, 0);
                })
            }
        }
    }

    private set_attribute(id: number, name: string, value: string) {
        this.nodes.get("set_attribute", id, (node) => {
            if (node instanceof Element) {
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
            } else {
                console.error("set_attribute error");
            }
        });
    }

    private remove_node(id: number) {
        this.nodes.delete("remove_node", id, (node) => {
            node.remove();
        });
    }

    private create_text(id: number, value: string) {
        const text = document.createTextNode(value);
        this.texts.set(id, text);
    }

    private remove_text(id: number) {
        this.texts.delete("remove_node", id, (text) => {
            text.remove();
        });
    }

    private update_text(id: number, value: string) {
        this.texts.get("set_attribute", id, (text) => {
            text.textContent = value;
        });
    }

    private get_node(label: string, id: number, callback: (node: Element | Comment | Text) => void) {
        const node = this.nodes.getItem(id);
        if (node !== undefined) {
            callback(node);
            return;
        }
        const text = this.texts.getItem(id);

        if (text !== undefined) {
            callback(text);
            return;
        }

        console.error(`${label}->get_node: Item id not found = ${id}`);
        return;
    }

    private insert_before(parent: number, child: number, ref_id: number | null | undefined) {
        this.nodes.get("insert_before", parent, (parentNode) => {
            this.get_node("insert_before child", child, (childNode) => {

                if (ref_id === null || ref_id === undefined) {
                    parentNode.insertBefore(childNode, null);
                } else {
                    this.get_node('insert_before ref', ref_id, (ref_node) => {
                        parentNode.insertBefore(childNode, ref_node);
                    });
                }
            });
        });
    }

    private insert_css(selector: string, value: string) {
        const style = document.createElement('style');
        const content = document.createTextNode(`${selector} { ${value} }`);
        style.appendChild(content);

        document.head.appendChild(style);
    }

    private callback_click(event: Event, callback_id: bigint) {
        event.preventDefault();
        this.getWasm().wasm_callback(callback_id, undefined);
    }

    private callback_input(event: Event, callback_id: bigint) {
        const target = event.target;

        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
            this.getWasm().wasm_callback(callback_id, target.value);
            return;
        }

        console.warn('event input ignore', target);
    }

    private callback_mouseenter(_event: Event, callback_id: bigint) {
        // event.preventDefault();
        this.getWasm().wasm_callback(callback_id, undefined);
    }

    private callback_mouseleave(_event: Event, callback_id: bigint) {
        // event.preventDefault();
        this.getWasm().wasm_callback(callback_id, undefined);
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
                            params.push([
                                file.name,
                                file.data,
                            ]);
                        }

                        this.getWasm().wasm_callback(callback_id, [params]);
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
            const result = this.getWasm().wasm_callback(callback_id, [
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

    private callback_add(id: number, event_name: string, callback_id: bigint) {
        const callback = (event: Event) => {
            if (event_name === 'click') {
                return this.callback_click(event, callback_id);
            }

            if (event_name === 'input') {
                return this.callback_input(event, callback_id);
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
            this.nodes.get('callback_add', id, (node) => {
                node.addEventListener(event_name, callback, false);
            });
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
            this.nodes.get('callback_remove', id, (node) => {
                node.removeEventListener(event_name, callback);
            });
        }
    }

    private clearRootContent = () => {
        while (true) {
            const first = this.root.firstChild;

            if (first === null) {
                return;
            } else {
                first.remove();
            }
        }
    }

    public dom_bulk_update = (value: string) => {
        if (this.initBeforeFirstUpdate === false) {
            this.initBeforeFirstUpdate = true;
            this.clearRootContent();
        }

        const setFocus: Set<number> = new Set();

        try {
            const commands: Array<CommandType> = JSON.parse(value);

            for (const command of commands) {
                try {
                    this.bulk_update_command(command);
                } catch (error) {
                    console.error('bulk_update - item', error, command);
                }

                if (command.type === 'set_attr' && command.name.toLocaleLowerCase() === 'autofocus') {
                    setFocus.add(command.id);
                }
            }
        } catch (error) {
            console.warn('buil_update - check in: https://jsonformatter.curiousconcept.com/')
            console.warn('bulk_update - param', value);
            console.error('bulk_update - incorrectly json data', error);
        }

        if (setFocus.size > 0) {
            setTimeout(() => {
                for (const id of setFocus) {
                    this.nodes.get(`set focus ${id}`, id, (node) => {
                        if (node instanceof HTMLElement) {
                            node.focus();
                        } else {
                            console.error('setfocus: HTMLElement expected');
                        }
                    });
                }
            }, 0);
        }
    }

    private bulk_update_command(command: CommandType) {
        if (command.type === 'remove_node') {
            this.remove_node(command.id);
            return;
        }

        if (command.type === 'insert_before') {
            this.insert_before(command.parent, command.child, command.ref_id === null ? null : command.ref_id);
            return;
        }

        if (command.type === 'create_node') {
            this.create_node(command.id, command.name);
            return;
        }

        if (command.type === 'create_text') {
            this.create_text(command.id, command.value);
            return;
        }

        if (command.type === 'update_text') {
            this.update_text(command.id, command.value);
            return;
        }

        if (command.type === 'set_attr') {
            this.set_attribute(command.id, command.name, command.value);
            return;
        }

        if (command.type === 'remove_text') {
            this.remove_text(command.id);
            return;
        }

        if (command.type === 'insert_css') {
            this.insert_css(command.selector, command.value);
            return;
        }

        if (command.type === 'create_comment') {
            const comment = document.createComment(command.value);
            this.nodes.set(command.id, comment);
            return;
        }

        if (command.type === 'remove_comment') {
            this.nodes.delete("remove_comment", command.id, (comment) => {
                comment.remove();
            });
            return;
        }

        if (command.type === 'callback_add') {
            this.callback_add(command.id, command.event_name, BigInt(command.callback_id));
            return;
        }

        if (command.type === 'callback_remove') {
            this.callback_remove(command.id, command.event_name, BigInt(command.callback_id));
            return;
        }

        return assertNeverCommand(command);
    }
}
