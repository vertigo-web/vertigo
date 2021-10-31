const createElement = (name) => {
    if (name == "path" || name == "svg") {
        return document.createElementNS("http://www.w3.org/2000/svg", name);
    }
    else {
        return document.createElement(name);
    }
};
class MapNodes {
    data;
    constructor() {
        this.data = new Map();
    }
    set(key, value) {
        this.data.set(key, value);
    }
    getItem(key) {
        return this.data.get(key);
    }
    mustGetItem(key) {
        const item = this.data.get(key);
        if (item === undefined) {
            throw Error(`item not found=${key}`);
        }
        return item;
    }
    get(label, key, callback) {
        const item = this.data.get(key);
        if (item === undefined) {
            console.error(`${label}->get: Item id not found = ${key}`);
        }
        else {
            callback(item);
        }
    }
    get2(label, key1, key2, callback) {
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
    delete(label, key, callback) {
        const item = this.data.get(key);
        this.data.delete(key);
        if (item === undefined) {
            console.error(`${label}->delete: Item id not found = ${key}`);
        }
        else {
            this.data.delete(key);
            callback(item);
        }
    }
}
export class DriverBrowserDomJs {
    mouse_down;
    mouse_over;
    keydown;
    oninput;
    nodes;
    texts;
    all;
    constructor(mouse_down, mouse_over, keydown, oninput) {
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
                this.mouse_over(id);
                return;
            }
            console.warn('mouseover ignore', target);
        }, false);
        document.addEventListener('keydown', (event) => {
            const target = event.target;
            if (target instanceof Element && event instanceof KeyboardEvent) {
                const id = this.all.get(target);
                const stopPropagate = this.keydown(id, event.key, event.code, event.altKey, event.ctrlKey, event.shiftKey, event.metaKey);
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
    mount_root(root_id) {
        this.nodes.get("append_to_body", root_id, (root) => {
            document.body.appendChild(root);
        });
    }
    create_node(id, name) {
        const node = createElement(name);
        this.nodes.set(id, node);
        this.all.set(node, id);
    }
    rename_node(id, name) {
        this.nodes.get("rename_node", id, (node) => {
            const new_node = createElement(name);
            while (true) {
                const firstChild = node.firstChild;
                if (firstChild) {
                    new_node.appendChild(firstChild);
                }
                else {
                    this.all.delete(node);
                    this.all.set(new_node, id);
                    this.nodes.set(id, new_node);
                    return;
                }
            }
        });
    }
    set_attribute(id, name, value) {
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
    remove_attribute(id, name) {
        this.nodes.get("remove_attribute", id, (node) => {
            node.removeAttribute(name);
        });
    }
    remove_node(id) {
        this.nodes.delete("remove_node", id, (node) => {
            this.all.delete(node);
            const parent = node.parentElement;
            if (parent !== null) {
                parent.removeChild(node);
            }
        });
    }
    create_text(id, value) {
        const text = document.createTextNode(value);
        this.texts.set(id, text);
        this.all.set(text, id);
    }
    remove_text(id) {
        this.texts.delete("remove_node", id, (text) => {
            this.all.delete(text);
            const parent = text.parentElement;
            if (parent !== null) {
                parent.removeChild(text);
            }
        });
    }
    update_text(id, value) {
        this.texts.get("set_attribute", id, (text) => {
            text.textContent = value;
        });
    }
    get_node(label, id, callback) {
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
    insert_before(parent, child, ref_id) {
        this.nodes.get("insert_before", parent, (parentNode) => {
            this.get_node("insert_before child", child, (childNode) => {
                if (ref_id === null || ref_id === undefined) {
                    parentNode.insertBefore(childNode, null);
                }
                else {
                    this.get_node('insert_before ref', ref_id, (ref_node) => {
                        parentNode.insertBefore(childNode, ref_node);
                    });
                }
            });
        });
    }
    insert_css(selector, value) {
        const style = document.createElement('style');
        const content = document.createTextNode(`${selector} { ${value} }`);
        style.appendChild(content);
        document.head.appendChild(style);
    }
    get_bounding_client_rect_x(node_id) {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().x;
    }
    get_bounding_client_rect_y(node_id) {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().y;
    }
    get_bounding_client_rect_width(node_id) {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().width;
    }
    get_bounding_client_rect_height(node_id) {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().height;
    }
    scroll_top(node_id) {
        return this.nodes.mustGetItem(node_id).scrollTop;
    }
    set_scroll_top(node_id, value) {
        this.nodes.mustGetItem(node_id).scrollTop = value;
    }
    scroll_left(node_id) {
        return this.nodes.mustGetItem(node_id).scrollLeft;
    }
    set_scroll_left(node_id, value) {
        return this.nodes.mustGetItem(node_id).scrollLeft = value;
    }
    scroll_width(node_id) {
        return this.nodes.mustGetItem(node_id).scrollWidth;
    }
    scroll_height(node_id) {
        return this.nodes.mustGetItem(node_id).scrollHeight;
    }
}
