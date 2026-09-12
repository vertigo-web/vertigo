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
    removeAttribute(name: string) { this.attributes.delete(name); }
    getAttributeNames() { return Array.from(this.attributes.keys()); }
    addEventListener(_event: string, _callback: (e: any) => void) { }
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
    console.log("\n--- Test hydration 1: Extra Nodes during SSR ---");
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
    console.log("\n--- Test hydration 2: Text Mismatch ---");
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
    console.log("\n--- Test hydration 3: Tag Name Mismatch ---");
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
    console.log("\n--- Test hydration 4: Attribute Mismatch ---");
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

// 5. SVG keeps its own casing
function testSvgHydration() {
    console.log("\n--- Test hydration 5: SVG tag names ---");
    clearBody();

    // DOM: <svg><path/><linearGradient/></svg>, as the HTML parser builds it from SSR markup.
    const svg = new MockSvgElement('svg');
    const path = new MockSvgElement('path');
    const gradient = new MockSvgElement('linearGradient');
    svg.appendChild(path);
    svg.appendChild(gradient);
    documentMock.body.appendChild(svg);

    const commands: CommandType[] = [
        { CreateNode: { id: 50, name: 'svg' } },
        { CreateNode: { id: 51, name: 'path' } },
        { CreateNode: { id: 52, name: 'linearGradient' } },
        { InsertBefore: { parent: 3, child: 50, ref_id: null } },
        { InsertBefore: { parent: 50, child: 51, ref_id: null } },
        { InsertBefore: { parent: 50, child: 52, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    const report = hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(50) as any === svg, "<svg> claimed, not rebuilt");
    assert(mapNodes.getAnyOption(51) as any === path, "<path> claimed");
    assert(mapNodes.getAnyOption(52) as any === gradient, "<linearGradient> claimed");
    assert(documentMock.body.childNodes.length === 1, "SVG subtree survived");
    assert(report.matched === report.hydratable, "every SVG vnode matched");
}

// 6. The batch never contained <body>
function testMissingRoot() {
    console.log("\n--- Test hydration 6: Missing <body> in the batch ---");
    clearBody();

    const survivor = new MockElement('DIV');
    documentMock.body.appendChild(survivor);

    // A partial batch: a subtree that was never attached to the document roots.
    const commands: CommandType[] = [
        { CreateNode: { id: 60, name: 'div' } },
        { CreateNode: { id: 61, name: 'a' } },
        { InsertBefore: { parent: 60, child: 61, ref_id: null } },
    ];

    const errors: string[] = [];
    const original = console.error;
    console.error = (...args: any[]) => { errors.push(String(args[0])); };

    let report;
    try {
        report = hydrate(commands, new MapNodes(), mockedApiLocation());
    } finally {
        console.error = original;
    }

    assert(report.rootFound === false, "rootFound is false");
    assert(report.matched === 0, "nothing matched");
    assert(errors.length === 1, "the cause was reported once");
    assert(
        errors[0] !== undefined && errors[0].includes("no <body>"),
        "the error names the missing root rather than printing a bare 0 %",
    );
    assert(documentMock.body.childNodes.length === 1, "hydration left the DOM alone");
}

// 7. Marker comments do not count against the score
function testMarkersAreNotCountedAgainstTheScore() {
    console.log("\n--- Test hydration 7: Marker comments ---");
    clearBody();

    // The server strips comments from its output, so only the <div> is in the DOM.
    const div = new MockElement('DIV');
    documentMock.body.appendChild(div);

    // id 71 is a `render_value` anchor: inserted, but never given a name or a value.
    const commands: CommandType[] = [
        { CreateNode: { id: 70, name: 'div' } },
        { InsertBefore: { parent: 3, child: 70, ref_id: null } },
        { InsertBefore: { parent: 3, child: 71, ref_id: null } },
    ];

    const report = hydrate(commands, new MapNodes(), mockedApiLocation());

    assert(report.hydratable === 1, "only the <div> is hydratable");
    assert(report.skipped === 1, "the marker is counted as skipped");
    assert(report.matched === 1, "the <div> matched");
    assert(report.matched === report.hydratable, "a fully matching batch scores 100%");
    assert(report.total > report.hydratable, "...even though the batch mentions more ids");
}

// 8. A subtree created and dropped inside the same batch
function testCreateThenRemoveInOneBatch() {
    console.log("\n--- Test hydration 8: Create-then-remove churn ---");
    clearBody();

    // The server only ever rendered these two - the churned node was never in its output.
    const first = new MockElement('DIV'); first.setAttribute('id', 'first');
    const second = new MockElement('SPAN'); second.setAttribute('id', 'second');
    documentMock.body.appendChild(first);
    documentMock.body.appendChild(second);

    const commands: CommandType[] = [
        { CreateNode: { id: 80, name: 'div' } },
        { InsertBefore: { parent: 3, child: 80, ref_id: null } },
        // A subscriber re-ran during the mount: this was built and then dropped.
        { CreateNode: { id: 81, name: 'p' } },
        { InsertBefore: { parent: 3, child: 81, ref_id: null } },
        { RemoveNode: { id: 81 } },
        { CreateNode: { id: 82, name: 'span' } },
        { InsertBefore: { parent: 3, child: 82, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    const report = hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(80) as any === first, "the <div> was claimed");
    assert(
        mapNodes.getAnyOption(82) as any === second,
        "the <span> after the churn was claimed, not deleted while scanning past a ghost",
    );
    assert(documentMock.body.childNodes.length === 2, "both server nodes survived");
    assert(report.matched === report.hydratable, "the removed node is not in the denominator");
}

// 9. UpdateText later in the same batch is the value that counts
function testUpdateTextWins() {
    console.log("\n--- Test hydration 9: UpdateText within the batch ---");
    clearBody();

    // What the server rendered - it replayed the whole batch, so it saw the final value.
    const textNode = new MockText("final");
    documentMock.body.appendChild(textNode);

    const commands: CommandType[] = [
        { CreateText: { id: 90, value: "stale" } },
        { InsertBefore: { parent: 3, child: 90, ref_id: null } },
        { UpdateText: { id: 90, value: "final" } },
    ];

    const mapNodes = new MapNodes();
    hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(90) as any === textNode, "the text node was claimed");
    assert(
        textNode.textContent === "final",
        "the server's text was kept, not overwritten with the value CreateText carried",
    );
}

// 10. Attributes the server rendered that this tree does not have
function testStaleAttributeRemoved() {
    console.log("\n--- Test hydration 10: Stale attribute on an adopted node ---");
    clearBody();

    // The server drew this anchor with an href; the browser's tree draws it without one.
    const anchor = new MockElement('A');
    anchor.setAttribute('href', '/server-only');
    anchor.setAttribute('class', 'old');
    documentMock.body.appendChild(anchor);

    const commands: CommandType[] = [
        { CreateNode: { id: 100, name: 'a' } },
        { SetAttr: { id: 100, name: 'class', value: 'new' } },
        { InsertBefore: { parent: 3, child: 100, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    hydrate(commands, mapNodes, mockedApiLocation());

    assert(mapNodes.getAnyOption(100) as any === anchor, "the anchor was adopted");
    assert(anchor.getAttribute('class') === 'new', "the tracked attribute was updated");
    assert(
        anchor.getAttribute('href') === undefined,
        "the server's href was dropped - adopting a node takes on its attributes in both directions",
    );
}

// Run all tests
testExtraNodes();
testTextMismatch();
testTagMismatch();
testAttributeMismatch();
testSvgHydration();
testMissingRoot();
testMarkersAreNotCountedAgainstTheScore();
testCreateThenRemoveInOneBatch();
testUpdateTextWins();
testStaleAttributeRemoved();
