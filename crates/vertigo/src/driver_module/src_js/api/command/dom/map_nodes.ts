type NodeType = Element | Comment | Text;
export class MapNodes {
    private data: Map<number, NodeType>;
    private initNodes: Array<ChildNode> | null;
    private style: HTMLStyleElement;

    constructor() {
        this.data = new Map();

        this.initNodes = [
            ...this.getRootHead().childNodes,
            ...this.getRootBody().childNodes,
        ];

        this.style = document.createElement('style');
    }

    private getRootHtml(): Element {
        return document.documentElement;
    }

    private getRootHead(): Element {
        return document.head;
    }

    private getRootBody(): Element {
        return document.body;
    }

    public set(id: number, value: NodeType) {
        if (id === 1 || id === 2 || id === 3) {
            //ignore
        } else {
            this.data.set(id, value);
        }
    }

    public getAnyOption(id: number): NodeType | undefined {
        if (id === 1) {
            return this.getRootHtml();
        }

        if (id === 2) {
            return this.getRootHead();
        }

        if (id === 3) {
            return this.getRootBody();
        }

        return this.data.get(id);
    }

    public getAny(label: string, id: number): NodeType {
        const item = this.getAnyOption(id);

        if (item === undefined) {
            throw Error(`${label} -> item not found=${id}`);
        }

        return item;
    }

    public get(label: string, id: number): NodeType {
        const item = this.getAnyOption(id);

        if (item === undefined) {
            throw new Error(`${label}->get: Item id not found = ${id}`);
        }
        return item;
    }

    public getNodeElement(label: string, id: number): HTMLElement {
        const node = this.get(label, id);
        if (node instanceof HTMLElement) {
            return node;
        } else {
            throw Error(`Expected id=${id} as HTMLElement`);
        }
    }

    public getNode(label: string, id: number): Element {
        const node = this.get(label, id);
        if (node instanceof Element) {
            return node;
        } else {
            throw Error(`Expected id=${id} as Element`);
        }
    }

    public getText(label: string, id: number): Text {
        const node = this.get(label, id);
        if (node instanceof Text) {
            return node;
        } else {
            throw Error(`Expected id=${id} as Text`);
        }
    }

    public getComment(label: string, id: number): Comment {
        const node = this.get(label, id);
        if (node instanceof Comment) {
            return node;
        } else {
            throw Error(`Expected id=${id} as Comment`);
        }
    }

    public delete(label: string, id: number): NodeType {
        const item = this.getAnyOption(id);
        this.data.delete(id);

        if (item === undefined) {
            throw new Error(`${label}->delete: Item id not found = ${id}`);
        }

        return item;
    }

    public insertCss(selector: string | null, value: string) {
        if (selector !== null) {
            // Add autocss styles
            const content = document.createTextNode(`\n${selector} { ${value} }`);
            this.style.appendChild(content);
        } else {
            // Add bundle (i.e. a tailwind bundle)
            const content = document.createTextNode(`\n${value}`);
            this.style.appendChild(content);
        }
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

    public insertBefore(parent: number, child: number, ref_id: number | null | undefined) {
        const parentNode = this.get("insert_before", parent);
        const childNode = this.getAny("insert_before child", child);

        if (ref_id === null || ref_id === undefined) {
            parentNode.insertBefore(childNode, null);
        } else {
            const ref_node = this.getAny('insert_before ref', ref_id);
            parentNode.insertBefore(childNode, ref_node);
        }
    }

    public addStyles() {
        this.getRootHead().appendChild(this.style);
    }

    public hasInitNodes(): boolean {
        return this.initNodes !== null;
    }

    public claimNode(id: number, node: NodeType) {
        this.data.set(id, node);

        if (this.initNodes) {
            const index = this.initNodes.indexOf(node as ChildNode);
            if (index > -1) {
                this.initNodes.splice(index, 1);
            }
        }
    }

    public has(id: number): boolean {
        return this.data.has(id);
    }
}
