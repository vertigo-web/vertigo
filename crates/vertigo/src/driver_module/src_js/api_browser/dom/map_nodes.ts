type NodeType = Element | Comment | Text;
export class MapNodes {
    private data: Map<number, NodeType>;
    private initNodes: Array<ChildNode> | null;
    private style: HTMLStyleElement;

    constructor() {
        this.data = new Map();

        this.initNodes = [
            ...this.get_root_head().childNodes,
            ...this.get_root_body().childNodes,
        ];

        this.style = document.createElement('style');
        this.get_root_head().appendChild(this.style);
    }

    private get_root_html(): Element {
        return document.documentElement;
    }

    private get_root_head(): Element {
        return document.head;
    }

    private get_root_body(): Element {
        return document.body;
    }

    public set(id: number, value: NodeType) {
        if (id === 1 || id === 2 || id === 3) {
            //ignore
        } else {
            this.data.set(id, value);
        }
    }

    public get_any_option(id: number): NodeType | undefined {
        if (id === 1) {
            return this.get_root_html();
        }

        if (id === 2) {
            return this.get_root_head();
        }

        if (id === 3) {
            return this.get_root_body();
        }

        return this.data.get(id);
    }

    public get_any(label: string, id: number): NodeType {
        const item = this.get_any_option(id);

        if (item === undefined) {
            throw Error(`${label} -> item not found=${id}`);
        }

        return item;
    }

    public get(label: string, id: number): NodeType {
        const item = this.get_any_option(id);

        if (item === undefined) {
            throw new Error(`${label}->get: Item id not found = ${id}`);
        }
        return item;
    }

    public get_node_element(label: string, id: number): HTMLElement {
        const node = this.get(label, id);
        if (node instanceof HTMLElement) {
            return node;
        } else {
            throw Error(`Expected id=${id} as HTMLElement`);
        }
    }

    public get_node(label: string, id: number): Element {
        const node = this.get(label, id);
        if (node instanceof Element) {
            return node;
        } else {
            throw Error(`Expected id=${id} as Element`);
        }
    }

    public get_text(label: string, id: number): Text {
        const node = this.get(label, id);
        if (node instanceof Text) {
            return node;
        } else {
            throw Error(`Expected id=${id} as Text`);
        }
    }

    public get_comment(label: string, id: number): Comment {
        const node = this.get(label, id);
        if (node instanceof Comment) {
            return node;
        } else {
            throw Error(`Expected id=${id} as Comment`);
        }
    }

    public delete(label: string, id: number): NodeType {
        const item = this.get_any_option(id);
        this.data.delete(id);

        if (item === undefined) {
            throw new Error(`${label}->delete: Item id not found = ${id}`);
        }

        return item;
    }

    public insert_css(selector: string, value: string) {
        const content = document.createTextNode(`\n${selector} { ${value} }`);
        this.style.appendChild(content);
    }

    public removeInitNodes() {
        const initNodes = this.initNodes;
        this.initNodes = null;

        if (initNodes === null) {
            return;
        }

        for (const node of initNodes) {
            node.remove();
        }
    }

    public insert_before(parent: number, child: number, ref_id: number | null | undefined) {
        const parentNode = this.get("insert_before", parent);
        const childNode = this.get_any("insert_before child", child);

        if (ref_id === null || ref_id === undefined) {
            parentNode.insertBefore(childNode, null);
        } else {
            const ref_node = this.get_any('insert_before ref', ref_id);
            parentNode.insertBefore(childNode, ref_node);
        }

        if (parentNode === this.get_root_head()) {
            //we make sure that the automatically generated styles are always the last element of the head
            this.get_root_head().appendChild(this.style);
        }
    }
}
