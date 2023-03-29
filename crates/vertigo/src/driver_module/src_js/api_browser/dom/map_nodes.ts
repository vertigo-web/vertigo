type NodeType = Element | Comment | Text;
export class MapNodes {
    private data: Map<number, NodeType>;

    constructor() {
        this.data = new Map();
    }

    public get_root_html(): Element {
        return document.documentElement;
    }

    public get_root_head(): Element {
        return document.head;
    }

    public get_root_body(): Element {
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
}
