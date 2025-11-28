// --- MOCKS ---
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
    addEventListener(_event: string, _callback: (e: any) => void) { }
}

class MockText extends MockNode {
    textContent: string;
    constructor(text: string) {
        super(MockNode.TEXT_NODE);
        this.textContent = text;
    }
}

class MockComment extends MockNode {
    textContent: string;
    constructor(text: string) {
        super(8);
        this.textContent = text;
    }
}

const documentMock = {
    body: new MockElement('BODY'),
    head: new MockElement('HEAD'),
    createElement: (tag: string) => new MockElement(tag),
    createTextNode: (text: string) => new MockText(text),
    createComment: (text: string) => new MockComment(text),
    documentElement: new MockElement('HTML'),
};

// Setup Global Mocks
(globalThis as any).Node = MockNode;
(globalThis as any).Element = MockElement;
(globalThis as any).Text = MockText;
(globalThis as any).Comment = MockComment;
(globalThis as any).document = documentMock;
(globalThis as any).window = {
    scrollTo: () => { }
};

// --- IMPORTS ---
// Must be imported AFTER mocks are set up if they have top-level side effects.
// However, standard imports are hoisted.
// We rely on map_nodes.ts and hydration.ts NOT having top-level side effects that use document
// OR that the side effects are inside classes/functions.
// MapNodes constructor uses document, so we must mock before instantiation.

import { hydrate } from "./hydration";
import { MapNodes } from "./map_nodes";
import { CommandType } from "./dom";

// --- TEST RUNNER ---
function assert(condition: boolean, message: string) {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
}

function clearBody() {
    const body = documentMock.body;
    while (body.childNodes.length > 0) {
        const child = body.childNodes[0];
        if (child) child.remove();
    }
}

function mockedApiLocation() {
    const mockAppLocation = {
        set: (_a: string, _b: string, _c: string) => { }
    } as any;
    return mockAppLocation;
}


// --- TESTS ---

// 1. Extra Nodes generated during SSR
function testExtraNodes() {
    console.log("\n--- Test 1: Extra Nodes during SSR ---");
    clearBody();

    // DOM: A, Extra, B
    const nodeA = new MockElement('DIV'); nodeA.setAttribute('id', 'A');
    const nodeExtra = new MockElement('SPAN'); nodeExtra.setAttribute('id', 'Extra');
    const nodeB = new MockElement('DIV'); nodeB.setAttribute('id', 'B');

    documentMock.body.appendChild(nodeA);
    documentMock.body.appendChild(nodeExtra);
    documentMock.body.appendChild(nodeB);

    // VDOM: A (id 10), B (id 11)
    const commands: CommandType[] = [
        { CreateNode: { id: 10, name: 'DIV' } },
        { CreateNode: { id: 11, name: 'DIV' } },
        { InsertBefore: { parent: 3, child: 10, ref_id: null } },
        { InsertBefore: { parent: 3, child: 11, ref_id: null } }
    ];

    const mapNodes = new MapNodes();
    hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(10) as any === nodeA, "Node A claimed");
    assert(mapNodes.getAnyOption(11) as any === nodeB, "Node B claimed");
    assert(documentMock.body.childNodes.length === 2, "Extra node removed");
    assert(documentMock.body.childNodes[0] === nodeA, "A is first");
    assert(documentMock.body.childNodes[1] === nodeB, "B is second");
}

// 2. Text node has different content
function testTextMismatch() {
    console.log("\n--- Test 2: Text Mismatch ---");
    clearBody();

    // DOM: "Old Text"
    const textNode = new MockText("Old Text");
    documentMock.body.appendChild(textNode);

    // VDOM: "New Text" (id 20)
    const commands: CommandType[] = [
        { CreateText: { id: 20, value: "New Text" } },
        { InsertBefore: { parent: 3, child: 20, ref_id: null } }
    ];

    const mapNodes = new MapNodes();

    hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(20) as any === textNode, "Text node claimed");
    assert(textNode.textContent === "New Text", "Text content updated");
}

// 3. Tag name is different
function testTagMismatch() {
    console.log("\n--- Test 3: Tag Name Mismatch ---");
    clearBody();

    // DOM: <SPAN>
    const spanNode = new MockElement('SPAN');
    documentMock.body.appendChild(spanNode);

    // VDOM: <DIV> (id 30)
    const commands: CommandType[] = [
        { CreateNode: { id: 30, name: 'DIV' } },
        { InsertBefore: { parent: 3, child: 30, ref_id: null } }
    ];

    const mapNodes = new MapNodes();

    hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(30) === undefined, "Node NOT claimed (mismatched tag)");
    assert(documentMock.body.childNodes.length === 0, "Mismatched node removed");
}

// 4. Element has different attributes
function testAttributeMismatch() {
    console.log("\n--- Test 4: Attribute Mismatch ---");
    clearBody();

    // DOM: <DIV class="old">
    const divNode = new MockElement('DIV');
    divNode.setAttribute('class', 'old');
    documentMock.body.appendChild(divNode);

    // VDOM: <DIV> (id 40) - Attributes should be checked by hydration
    const commands: CommandType[] = [
        { CreateNode: { id: 40, name: 'DIV' } },
        { SetAttr: { id: 40, name: 'class', value: 'new' } },
        { InsertBefore: { parent: 3, child: 40, ref_id: null } }
    ];

    const mapNodes = new MapNodes();

    hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(40) as any === divNode, "Node claimed despite attribute mismatch");
    assert(divNode.getAttribute('class') === 'new', "Attributes updated");
}

// Run all tests
testExtraNodes();
testTextMismatch();
testTagMismatch();
testAttributeMismatch();
