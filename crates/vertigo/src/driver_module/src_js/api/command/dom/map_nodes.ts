type NodeType = Element | Comment | Text;

/// The parts of a `MutationRecord` that tell what took a text's place.
export interface RemovalRecord {
    removedNodes: Iterable<Node>;
    addedNodes: Iterable<Node>;
    previousSibling: Node | null;
}

/// The changes made to the page from the moment the script runs until the app starts.
let earlyChanges: { observer: MutationObserver, records: Array<MutationRecord> } | null = null;

/// Starts noting the changes made to the page, from the moment the script runs until the app
/// starts - until then the page is the server's alone, so all of them come from something else.
/// `MapNodes` takes the notes over, and hydration knows from them what took the place of
/// a text the server sent (see `MapNodes::noticeReplacedBeforeStart`).
export const watchPage = () => {
    if (earlyChanges !== null || typeof MutationObserver === 'undefined') {
        return;
    }

    const records: Array<MutationRecord> = [];
    const observer = new MutationObserver((batch) => records.push(...batch));
    observer.observe(document.documentElement, { childList: true, subtree: true });
    earlyChanges = { observer, records };
};

const takeEarlyChanges = (): Array<MutationRecord> => {
    if (earlyChanges === null) {
        return [];
    }

    const { observer, records } = earlyChanges;
    earlyChanges = null;
    records.push(...observer.takeRecords());
    observer.disconnect();
    return records;
};

/// The node that took the place of a text `record` says was removed: the one added by the same
/// change (`replaceChild`), or the one put right before the text first (`insertBefore`, then
/// `removeChild`). Only an element counts, and only one `isOther` says isn't the app's.
const standInOf = (record: RemovalRecord, isOther: (node: Element) => boolean): Element | undefined =>
    [...record.addedNodes, record.previousSibling]
        .find((node): node is Element => node instanceof Element && node.isConnected && isOther(node));

export class MapNodes {
    private data: Map<number, NodeType>;
    private initNodes: Array<ChildNode> | null;
    private style: HTMLStyleElement;

    /// Every node the app has, to tell them from the ones something else put in the page.
    private owned: WeakSet<Node> = new WeakSet();

    /// Texts of the app something else took out of the page, and the nodes it put in their
    /// place.
    ///
    /// A page translator - Google's, the one built into Chrome - replaces each text with
    /// a `<font>` holding the translation, and a browser extension or another script on the page
    /// can do the like. The app keeps the text it has rendered, now out of the document, so
    /// a change to it would never show: a text with the change goes in place of its stand-in
    /// instead (see `textInPlace`), for whatever replaced the text to take it on anew.
    private standIns: WeakMap<Text, ChildNode> = new WeakMap();

    /// Nodes put in place of a text before the app started - where hydration looks for a text
    /// the server sent.
    private replacedBeforeStart: WeakSet<Node> = new WeakSet();

    constructor() {
        this.data = new Map();

        this.initNodes = [
            ...this.getRootHead().childNodes,
            ...this.getRootBody().childNodes,
        ];

        this.style = document.createElement('style');

        this.noticeReplacedBeforeStart(takeEarlyChanges());

        // Kept by the document it watches, for as long as the page lasts
        if (typeof MutationObserver !== 'undefined') {
            new MutationObserver((records) => this.noticeRemoved(records))
                .observe(this.getRootHtml(), { childList: true, subtree: true });
        }
    }

    /// Notes the nodes that took the place of a text before the app started (see `watchPage`):
    /// ones added by then, as everything the server sent was in the page before the script ran.
    public noticeReplacedBeforeStart(records: Iterable<RemovalRecord>) {
        const changes = [...records];
        const added = new Set(changes.flatMap((record) => [...record.addedNodes]));

        for (const record of changes) {
            for (const removed of record.removedNodes) {
                if (!(removed instanceof Text) || removed.isConnected) {
                    continue;
                }

                const standIn = standInOf(record, (node) => added.has(node));
                if (standIn) {
                    this.replacedBeforeStart.add(standIn);
                }
            }
        }
    }

    /// Whether `node` was put in place of a text before the app started.
    public replacesText(node: Node): boolean {
        return this.replacedBeforeStart.has(node);
    }

    /// Notes what took the place of a text of the app that something else removed.
    ///
    /// Only texts are followed: an element is left where it was, with the texts in it replaced
    /// one by one. A text the app moved or removed itself is either back in the document by the
    /// time the records come, or no longer the app's.
    public noticeRemoved(records: Iterable<RemovalRecord>) {
        for (const record of records) {
            for (const removed of record.removedNodes) {
                if (!(removed instanceof Text) || !this.owned.has(removed) || removed.isConnected) {
                    continue;
                }

                const standIn = standInOf(record, (node) => !this.owned.has(node));
                if (standIn) {
                    this.standIns.set(removed, standIn);
                }
            }
        }
    }

    /// The node standing in for a text of the app that something else took out of the page, if
    /// it is still in it.
    ///
    /// The records of a change reach `noticeRemoved` at the end of the task it was made in, and
    /// the app changes the page in tasks of its own, so they are in by then.
    public standInFor(node: Node): ChildNode | null {
        if (!(node instanceof Text) || node.isConnected) {
            return null;
        }

        const standIn = this.standIns.get(node);
        return standIn?.isConnected ? standIn : null;
    }

    /// Marks a text hydration adopted with the node that had taken its place before the app
    /// started.
    public setStandIn(text: Text, standIn: ChildNode) {
        this.standIns.set(text, standIn);
    }

    /// Puts the text `id` back in place of its stand-in, after the app changed it.
    public textInPlace(id: number, text: Text) {
        const standIn = this.standInFor(text);
        if (standIn !== null) {
            standIn.replaceWith(this.renewed(id, text));
        }
    }

    /// A new node in place of a text something else took out of the page, under the same id.
    /// Not the text itself: whatever took it out may know that one as done with - a translator
    /// as translated - and leave it as it is once it's back.
    private renewed(id: number, text: Text): Text {
        const renewed = document.createTextNode(text.textContent);
        this.standIns.delete(text);
        this.owned.delete(text);
        this.set(id, renewed);
        return renewed;
    }

    /// Removes a node, or the stand-in of a text something else took out of the page.
    public removeFromPage(node: ChildNode) {
        this.standInFor(node)?.remove();
        node.remove();
    }

    /// What is in the page for a node: the node itself, or the stand-in of a text something
    /// else took out of it.
    public inPage(node: NodeType): ChildNode {
        return this.standInFor(node) ?? node;
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
            this.owned.add(value);
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

    // `expected` is passed rather than read off `kind.name` so the thrown text stays
    // greppable from source.
    //
    // `getNode` runs once per SetAttr and per RemoveAttr, so the `instanceof` against a
    // variable here is on a hot path. Checked on dom-bench, three interleaved rounds against
    // the three-literal-methods version: no cost - if anything slightly faster, and well
    // inside the noise.
    private expect<T>(label: string, id: number, kind: new () => T, expected: string): T {
        const node = this.get(label, id);
        if (node instanceof kind) {
            return node;
        } else {
            throw Error(`Expected id=${id} as ${expected}`);
        }
    }

    public getNode(label: string, id: number): Element {
        return this.expect(label, id, Element, "Element");
    }

    public getText(label: string, id: number): Text {
        return this.expect(label, id, Text, "Text");
    }

    public delete(label: string, id: number): NodeType {
        const item = this.getAnyOption(id);
        this.data.delete(id);

        if (item === undefined) {
            throw new Error(`${label}->delete: Item id not found = ${id}`);
        }

        this.owned.delete(item);
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
        let childNode = this.getAny("insert_before child", child);

        // A text something else took out of the page goes back as the app has it, for whatever
        // replaced it to take it on where it lands - its stand-in would stay behind otherwise.
        const standIn = this.standInFor(childNode);
        if (standIn !== null && childNode instanceof Text) {
            standIn.remove();
            childNode = this.renewed(child, childNode);
        }

        if (ref_id === null || ref_id === undefined) {
            parentNode.insertBefore(childNode, null);
        } else {
            const ref_node = this.inPage(this.getAny('insert_before ref', ref_id));
            parentNode.insertBefore(childNode, ref_node);
        }
    }

    public addStyles() {
        const head = this.getRootHead();

        // `appendChild` moves a node that is already in place too, and taking the styles out and
        // putting them back made the browser restyle the page after every batch.
        if (head.lastChild !== this.style) {
            head.appendChild(this.style);
        }
    }

    public hasInitNodes(): boolean {
        return this.initNodes !== null;
    }

    public claimNode(id: number, node: NodeType) {
        this.data.set(id, node);
        this.owned.add(node);

        if (this.initNodes) {
            const index = this.initNodes.indexOf(node);
            if (index > -1) {
                this.initNodes.splice(index, 1);
            }
        }
    }

    public has(id: number): boolean {
        // Root nodes always exist in real DOM
        if (id === 1 || id === 2 || id === 3) {
            return true;
        }

        return this.data.has(id);
    }
}
