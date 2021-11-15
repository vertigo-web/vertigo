const createElement = (name: string): Element => {
    if (name == "path" || name == "svg") {
        return document.createElementNS("http://www.w3.org/2000/svg", name);
    } else {
        return document.createElement(name);
    }
}

type CommandType = {
    type: 'mount_node'
    id: number,
} | {
    type: 'create_node',
    id: number,
    name: string,
} | {
    type: 'rename_node',
    id: number,
    new_name: string,
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
    key: string,
    value: string
} | {
    type: 'remove_attr',
    id: number,
    name: string,
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
};

const assertNeverCommand = (data: never): never => {
    console.error(data);
    throw Error('unknown command');
};

class MapNodes<K, V> {
    private data: Map<K, V>;

    constructor() {
        this.data = new Map();
    }

    public set(key: K, value: V) {
        this.data.set(key, value);
    }

    public getItem(key: K): V | undefined {
        return this.data.get(key);
    }

    public mustGetItem(key: K): V {
        const item = this.data.get(key);

        if (item === undefined) {
            throw Error(`item not found=${key}`);
        }

        return item;
    }

    public get(label: string, key: K, callback: (value: V) => void) {
        const item = this.data.get(key);

        if (item === undefined) {
            console.error(`${label}->get: Item id not found = ${key}`);
        } else {
            callback(item);
        }
    }

    public get2(label: string, key1: K, key2: K, callback: (node1: V, node2: V) => void) {
        const node1 = this.data.get(key1);
        const node2 = this.data.get(key2);

        if (node1 === undefined) {
            console.error(`${label}->get: Item id not found = ${key1}`);
            return;
        }

        if (node2 === undefined) {
            console.error(`${label}->get: Item id not found = ${key2}`);
            return;
        }

        callback(node1, node2);
    }

    public delete(label: string, key: K, callback: (value: V) => void) {
        const item = this.data.get(key);
        this.data.delete(key);

        if (item === undefined) {
            console.error(`${label}->delete: Item id not found = ${key}`);
        } else {
            this.data.delete(key);
            callback(item);
        }
    }
}

type KeydownCallbackType = (
    dom_id: BigInt | null,
    key: string,
    code: string,
    alt_key: boolean,
    ctrl_key: boolean,
    shift_key: boolean,
    meta_key: boolean
) => boolean;

export class DriverBrowserDomJs {
    private readonly mouse_down: (dom_id: BigInt) => void;
    private readonly mouse_over: (dom_id: BigInt | null) => void;
    private readonly keydown: KeydownCallbackType;
    private readonly oninput: (dom_id: BigInt, text: string) => void;
    private readonly nodes: MapNodes<BigInt, Element>;
    private readonly texts: MapNodes<BigInt, Text>;
    private readonly all: Map<Element | Text, BigInt>;

    public constructor(
        mouse_down: (dom_id: BigInt) => void,
        mouse_over: (dom_id: BigInt | null) => void,
        keydown: KeydownCallbackType,
        oninput: (dom_id: BigInt, text: string) => void,
    ) {
        this.mouse_down = mouse_down;
        this.mouse_over = mouse_over;
        this.keydown = keydown;
        this.oninput = oninput;
        this.nodes = new MapNodes();
        this.texts = new MapNodes();
        this.all = new Map();

        document.addEventListener('mousedown', (event) => {
            const target = event.target;

            if (target instanceof Element) {
                const id = this.all.get(target);

                if (id !== undefined) {
                    this.mouse_down(id);
                    return;
                }
            }

            console.warn('mousedown ignore', target);
        }, false);

        document.addEventListener('mouseover', (event) => {
            const target = event.target;

            if (target instanceof Element) {
                const id = this.all.get(target);

                if (id === undefined) {
                    this.mouse_over(null);
                    return;
                }

                this.mouse_over(id);
                return;
            }

            console.warn('mouseover ignore', target);
        }, false);

        document.addEventListener('keydown', (event) => {
            const target = event.target;

            if (target instanceof Element && event instanceof KeyboardEvent) {
                const id = this.all.get(target);

                const stopPropagate = this.keydown(
                    id === undefined ? null : id,
                    event.key,
                    event.code,
                    event.altKey,
                    event.ctrlKey,
                    event.shiftKey,
                    event.metaKey,
                );

                if (stopPropagate) {
                    event.preventDefault();
                    event.stopPropagation();
                }
                return;
            }

            console.warn('keydown ignore', target);
        }, false);

        document.addEventListener('input', (event) => {
            const target = event.target;

            if (target instanceof Element) {
                const id = this.all.get(target);

                if (id !== undefined) {
                    if (target instanceof HTMLInputElement) {
                        this.oninput(id, target.value);
                        return;
                    }

                    if (target instanceof HTMLTextAreaElement) {
                        this.oninput(id, target.value);
                        return;
                    }

                    return;
                }
            }

            console.warn('mouseover ignore', target);
        }, false);
    }

    private mount_node(root_id: BigInt) {
        this.nodes.get("append_to_body", root_id, (root) => {
            document.body.appendChild(root);
        });
    }

    private create_node(id: BigInt, name: string) {
        const node = createElement(name);
        this.nodes.set(id, node);
        this.all.set(node, id);
    }

    private rename_node(id: BigInt, name: string) {
        this.nodes.get("rename_node", id, (node) => {
            const new_node = createElement(name);

            while (true) {
                const firstChild = node.firstChild;

                if (firstChild) {
                    new_node.appendChild(firstChild);
                } else {
                    this.all.delete(node);
                    this.all.set(new_node, id);
                    this.nodes.set(id, new_node);
                    return;
                }
            }
        });
    }

    private set_attribute(id: BigInt, name: string, value: string) {
        this.nodes.get("set_attribute", id, (node) => {
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
        });
    }

    private remove_attribute(id: BigInt, name: string) {
        this.nodes.get("remove_attribute", id, (node) => {
            node.removeAttribute(name);
        });
    }

    private remove_node(id: BigInt) {
        this.nodes.delete("remove_node", id, (node) => {
            this.all.delete(node);

            const parent = node.parentElement;
            if (parent !== null) {
                parent.removeChild(node);
            }
        });
    }

    private create_text(id: BigInt, value: string) {
        const text = document.createTextNode(value);
        this.texts.set(id, text);
        this.all.set(text, id);
    }

    private remove_text(id: BigInt) {
        this.texts.delete("remove_node", id, (text) => {
            this.all.delete(text);

            const parent = text.parentElement;
            if (parent !== null) {
                parent.removeChild(text);
            }
        });
    }

    private update_text(id: BigInt, value: string) {
        this.texts.get("set_attribute", id, (text) => {
            text.textContent = value;
        });
    }

    private get_node(label: string, id: BigInt, callback: (node: Element | Text) => void) {
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

    private insert_before(parent: BigInt, child: BigInt, ref_id: BigInt | null | undefined) {
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

    public bulk_update(value: string) {
        try {
            const commands: Array<CommandType> = JSON.parse(value);

            for (const command of commands) {
                this.bulk_update_command(command);
            }
        } catch (error) {
            console.warn('buil_update - check in: https://jsonformatter.curiousconcept.com/')
            console.warn('bulk_update - param', value);
            console.error('bulk_update - incorrectly json data', error);
        }
    }

    private bulk_update_command(command: CommandType) {
        if (command.type === 'remove_node') {
            this.remove_node(BigInt(command.id));
            return;
        }

        if (command.type === 'insert_before') {
            this.insert_before(BigInt(command.parent), BigInt(command.child), command.ref_id === null ? null : BigInt(command.ref_id));
            return;
        }

        if (command.type === 'mount_node') {
            this.mount_node(BigInt(command.id));
            return;
        }

        if (command.type === 'create_node') {
            this.create_node(BigInt(command.id), command.name);
            return;
        }

        if (command.type === 'rename_node') {
            this.rename_node(BigInt(command.id), command.new_name);
            return;
        }

        if (command.type === 'create_text') {
            this.create_text(BigInt(command.id), command.value);
            return;
        }

        if (command.type === 'update_text') {
            this.update_text(BigInt(command.id), command.value);
            return;
        }

        if (command.type === 'set_attr') {
            this.set_attribute(BigInt(command.id), command.key, command.value);
            return;
        }

        if (command.type === 'remove_attr') {
            this.remove_attribute(BigInt(command.id), command.name);
            return;
        }

        if (command.type === 'remove_text') {
            this.remove_text(BigInt(command.id));
            return;
        }

        if (command.type === 'insert_css') {
            this.insert_css(command.selector, command.value);
            return;
        }

        return assertNeverCommand(command);
    }

    public get_bounding_client_rect_x(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().x;
    }

    public get_bounding_client_rect_y(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().y;
    }

    public get_bounding_client_rect_width(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().width;
    }

    public get_bounding_client_rect_height(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().height;
    }

    public scroll_top(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).scrollTop;
    }

    public set_scroll_top(node_id: BigInt, value: number) {
        this.nodes.mustGetItem(node_id).scrollTop = value;
    }

    public scroll_left(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).scrollLeft;
    }

    public set_scroll_left(node_id: BigInt, value: number) {
        return this.nodes.mustGetItem(node_id).scrollLeft = value;
    }

    public scroll_width(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).scrollWidth;
    }

    public scroll_height(node_id: BigInt): number {
        return this.nodes.mustGetItem(node_id).scrollHeight;
    }
}
