import { buildSnapshot } from './snapshot';

// --- MOCKS: copied from hydration.test.ts ---
class MockNode {
    static ELEMENT_NODE = 1;
    static TEXT_NODE = 3;
    nodeType: number;
    childNodes: MockNode[];
    parentNode: MockNode | null;

    constructor(type: number) {
        this.nodeType = type;
        this.childNodes = [];
        this.parentNode = null;
    }
    remove() {
        if (this.parentNode) {
            const idx = this.parentNode.childNodes.indexOf(this);
            if (idx > -1) this.parentNode.childNodes.splice(idx, 1);
            this.parentNode = null;
        }
    }
    appendChild(child: MockNode) {
        if (child.parentNode) child.remove();
        child.parentNode = this;
        this.childNodes.push(child);
    }
    get firstChild(): MockNode | null { return this.childNodes[0] || null; }
    get nextSibling(): MockNode | null {
        if (!this.parentNode) return null;
        const idx = this.parentNode.childNodes.indexOf(this);
        return this.parentNode.childNodes[idx + 1] || null;
    }
}

class MockElement extends MockNode {
    tagName: string;
    attributes: Map<string, string>;

    constructor(tagName: string) {
        super(MockNode.ELEMENT_NODE);
        this.tagName = tagName.toUpperCase();
        this.attributes = new Map();
    }
    setAttribute(name: string, value: string) { this.attributes.set(name, value); }
    getAttribute(name: string) { return this.attributes.get(name); }
    removeAttribute(name: string) { this.attributes.delete(name); }
    getAttributeNames() { return Array.from(this.attributes.keys()); }
    addEventListener(_event: string, _callback: (e: any) => void) { }
    hasAttribute(name: string) { return this.attributes.has(name); }
}

/// An element in the SVG namespace.
///
/// The distinction that matters here is the only one hydration can see: `createElementNS`
/// keeps the case it was given, and the HTML parser adjusts server-rendered SVG the same way,
/// so these report `tagName` as "svg" / "path" / "linearGradient" - never uppercased.
class MockSvgElement extends MockElement {
    constructor(tagName: string) {
        super(tagName);
        this.tagName = tagName;
    }
}

class MockText extends MockNode {
    data: string;
    constructor(data: string) {
        super(MockNode.TEXT_NODE);
        this.data = data;
    }
}

class MockComment extends MockNode {
    static COMMENT_NODE = 8;
    data: string;
    constructor(data: string) {
        super(MockComment.COMMENT_NODE);
        this.data = data;
    }
}

const assert = (condition: boolean, message: string) => {
    if (!condition) {
        throw new Error(message);
    }
};

const elementName = (node: any): string => ('Element' in node ? node.Element.name : '');

const names = (payload: any): Array<string> =>
    payload.nodes.filter((node: any) => 'Element' in node).map(elementName);

/// `<html lang="pl"><head><title>a</title></head><body><div class="x">hi</div></body></html>`
const document = () => {
    const html = new MockElement('html');
    html.setAttribute('lang', 'pl');

    const head = new MockElement('head');
    const title = new MockElement('title');
    title.appendChild(new MockText('a'));
    head.appendChild(title);

    const body = new MockElement('body');
    const div = new MockElement('div');
    div.setAttribute('class', 'x');
    div.appendChild(new MockText('hi'));
    body.appendChild(div);

    html.appendChild(head);
    html.appendChild(body);

    return html;
};

const run = () => {
    {
        const { payload, nodes } = buildSnapshot(document() as any);

        assert(payload.nodes.length === nodes.length, 'the payload and the node table must line up');
        assert(payload.head === 1, `head should be index 1, got ${payload.head}`);
        assert(payload.body === 3, `body should be index 3, got ${payload.body}`);

        const root = payload.nodes[0]!;
        assert(root !== undefined, 'root element must exist');
        assert(elementName(root) === 'html', 'index 0 is <html>');
        assert(
            'Element' in root && root.Element.attrs.some(attr => attr.name === 'lang' && attr.value === 'pl'),
            'attributes travel with the element',
        );
        assert(
            'Element' in root && root.Element.children.length === 2,
            'children are recorded as indices into the same list',
        );
        const titleText = payload.nodes[5]!;
        assert(titleText !== undefined && 'Text' in titleText, 'the title text is at index 5');
    }

    {
        // tagName is uppercase for html and preserves case for svg; rust compares
        // case-insensitively, so we send everything lowercase.
        const html = new MockElement('html');
        const body = new MockElement('body');
        const svg = new MockSvgElement('svg');
        svg.appendChild(new MockSvgElement('linearGradient'));
        body.appendChild(svg);
        html.appendChild(body);

        const { payload } = buildSnapshot(html as any);

        assert(names(payload).includes('svg'), `expected svg among ${names(payload).join()}`);
        assert(
            names(payload).includes('lineargradient'),
            `expected a lowercased svg name among ${names(payload).join()}`,
        );
    }

    {
        // Whitespace from formatting goes to rust - js cannot safely filter it,
        // because in <pre> and with inline content it is significant. The decision belongs to rust.
        const html = new MockElement('html');
        const body = new MockElement('body');
        body.appendChild(new MockText('\n  '));
        body.appendChild(new MockElement('div'));
        html.appendChild(body);

        const { payload } = buildSnapshot(html as any);

        const texts = payload.nodes.filter(node => 'Text' in node);
        assert(texts.length === 1, `whitespace text nodes must be sent, got ${texts.length}`);
    }

    {
        // Comments too, if only so rust knows that something occupies the slot.
        const html = new MockElement('html');
        const body = new MockElement('body');
        body.appendChild(new MockComment('hand written'));
        html.appendChild(body);

        const { payload } = buildSnapshot(html as any);

        assert(
            payload.nodes.some(node => 'Comment' in node),
            'comments must be sent',
        );
    }

    {
        // The script loading wasm is not part of the application tree.
        const html = new MockElement('html');
        const body = new MockElement('body');
        body.appendChild(new MockElement('div'));
        const script = new MockElement('script');
        script.setAttribute('data-vertigo-run-wasm', 'x');
        body.appendChild(script);
        html.appendChild(body);

        const { payload, nodes } = buildSnapshot(html as any);

        assert(!names(payload).includes('script'), `the loader script must be skipped, got ${names(payload).join()}`);
        assert(payload.nodes.length === nodes.length, 'skipping must not desynchronise the two lists');
    }

    console.info('snapshot.test.ts: ok');
};

run();
