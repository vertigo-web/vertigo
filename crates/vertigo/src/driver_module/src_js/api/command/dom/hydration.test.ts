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
        if (child.parentNode) {
            MockNode.moved.push(child);
            child.remove();
        }
        child.parentNode = this;
        this.childNodes.push(child);
    }
    /// Nodes `insertBefore` or `appendChild` took out of a parent to put them back - which in a
    /// browser drops their focus, their scroll position, an iframe's page, and for a `<style>`
    /// restyles the page.
    static moved: MockNode[] = [];
    insertBefore(child: MockNode, ref: MockNode | null) {
        if (child.parentNode) {
            MockNode.moved.push(child);
            child.remove();
        }
        child.parentNode = this;
        const index = ref ? this.childNodes.indexOf(ref) : -1;
        if (index === -1) this.childNodes.push(child);
        else this.childNodes.splice(index, 0, child);
        return child;
    }
    get firstChild(): MockNode | null { return this.childNodes[0] || null; }
    get lastChild(): MockNode | null { return this.childNodes[this.childNodes.length - 1] || null; }
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
    const { report } = hydrate(commands, mapNodes, mockedApiLocation());

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
        report = hydrate(commands, new MapNodes(), mockedApiLocation()).report;
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

    const { report } = hydrate(commands, new MapNodes(), mockedApiLocation());

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
    const { report } = hydrate(commands, mapNodes, mockedApiLocation());

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

/// Runs `hydrate`, collecting what it logged with `console.error`.
function hydrateCollectingErrors(commands: CommandType[], mapNodes: MapNodes) {
    const errors: string[] = [];
    const original = console.error;
    console.error = (...args: any[]) => { errors.push(String(args[0])); };

    try {
        const { report } = hydrate(commands, mapNodes, mockedApiLocation());
        return { report, errors };
    } finally {
        console.error = original;
    }
}

// 11. An empty text has nothing to match
function testEmptyTextIsNotCountedAgainstTheScore() {
    console.log("\n--- Test hydration 11: Empty text ---");
    clearBody();

    // `<a><span>{label}</span><span>" >"</span></a>` with `label == ""`: the server writes
    // `<span></span>`, as there is no way to write an empty text node in HTML.
    const anchor = new MockElement('A');
    const label = new MockElement('SPAN');
    const symbol = new MockElement('SPAN');
    const symbolText = new MockText(" >");
    symbol.appendChild(symbolText);
    anchor.appendChild(label);
    anchor.appendChild(symbol);
    documentMock.body.appendChild(anchor);

    const commands: CommandType[] = [
        { CreateNode: { id: 110, name: 'a' } },
        { CreateNode: { id: 111, name: 'span' } },
        { CreateText: { id: 112, value: "" } },
        { CreateNode: { id: 113, name: 'span' } },
        { CreateText: { id: 114, value: " >" } },
        { InsertBefore: { parent: 3, child: 110, ref_id: null } },
        { InsertBefore: { parent: 110, child: 111, ref_id: null } },
        { InsertBefore: { parent: 111, child: 112, ref_id: null } },
        { InsertBefore: { parent: 110, child: 113, ref_id: null } },
        { InsertBefore: { parent: 113, child: 114, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    const { report, errors } = hydrateCollectingErrors(commands, mapNodes);

    assert(mapNodes.getAnyOption(111) as any === label, "the empty <span> was claimed");
    assert(mapNodes.getAnyOption(114) as any === symbolText, "the text after it was claimed");
    assert(mapNodes.getAnyOption(112) === undefined, "the empty text is left to the replay");
    assert(report.skipped === 1, "the empty text is counted as skipped");
    assert(report.matched === report.hydratable, "a batch with an empty text scores 100%");
    assert(errors.length === 0, "nothing was reported as a mismatch");
}

// 12. An empty text does not take over the text after its siblings
function testEmptyTextDoesNotEatItsSiblings() {
    console.log("\n--- Test hydration 12: Empty text before an element ---");
    clearBody();

    // `<div>{""}<b>"bold"</b>"tail"</div>` - the server wrote `<div><b>bold</b>tail</div>`.
    const div = new MockElement('DIV');
    const bold = new MockElement('B');
    bold.appendChild(new MockText("bold"));
    const tail = new MockText("tail");
    div.appendChild(bold);
    div.appendChild(tail);
    documentMock.body.appendChild(div);

    const commands: CommandType[] = [
        { CreateNode: { id: 120, name: 'div' } },
        { CreateText: { id: 121, value: "" } },
        { CreateNode: { id: 122, name: 'b' } },
        { CreateText: { id: 123, value: "bold" } },
        { CreateText: { id: 124, value: "tail" } },
        { InsertBefore: { parent: 3, child: 120, ref_id: null } },
        { InsertBefore: { parent: 120, child: 121, ref_id: null } },
        { InsertBefore: { parent: 120, child: 122, ref_id: null } },
        { InsertBefore: { parent: 122, child: 123, ref_id: null } },
        { InsertBefore: { parent: 120, child: 124, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    const { report, errors } = hydrateCollectingErrors(commands, mapNodes);

    assert(mapNodes.getAnyOption(122) as any === bold, "the <b> was claimed, not deleted");
    assert(mapNodes.getAnyOption(124) as any === tail, "\"tail\" was claimed by its own vnode");
    assert(tail.textContent === "tail", "...and kept its text");
    assert(div.childNodes.length === 2, "both server nodes survived");
    assert(report.matched === report.hydratable, "every non-empty vnode matched");
    assert(errors.length === 0, "nothing was reported as a mismatch");
}

// 13. An empty text inside texts the server merged
function testEmptyTextInsideMergedTexts() {
    console.log("\n--- Test hydration 13: Empty text between texts ---");
    clearBody();

    // `<p>"abc"{""}"def"</p>` - the server merged the texts into one node.
    const paragraph = new MockElement('P');
    const merged = new MockText("abcdef");
    paragraph.appendChild(merged);
    documentMock.body.appendChild(paragraph);

    const commands: CommandType[] = [
        { CreateNode: { id: 130, name: 'p' } },
        { CreateText: { id: 131, value: "abc" } },
        { CreateText: { id: 132, value: "" } },
        { CreateText: { id: 133, value: "def" } },
        { InsertBefore: { parent: 3, child: 130, ref_id: null } },
        { InsertBefore: { parent: 130, child: 131, ref_id: null } },
        { InsertBefore: { parent: 130, child: 132, ref_id: null } },
        { InsertBefore: { parent: 130, child: 133, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    const { report, errors } = hydrateCollectingErrors(commands, mapNodes);

    assert(mapNodes.getAnyOption(131) as any === merged, "the merged text went to the first vnode");
    assert(report.skipped === 1, "only the empty text is skipped");
    assert(report.matched === report.hydratable, "the empty text did not break up the group");
    assert(errors.length === 0, "nothing was reported as a mismatch");
}

// 14. Server markup the browser's tree has no place for
function testDiscardedServerNodesAreCounted() {
    console.log("\n--- Test hydration 14: Discarded server nodes ---");
    clearBody();

    // The server rendered a list, indented as its HTML is, and the script that starts the app:
    // <div>⏎<div>Apples</div>⏎<div>Bread</div></div><script>
    const list = new MockElement('DIV');
    list.appendChild(new MockText("\n  "));
    const apples = new MockElement('DIV');
    apples.appendChild(new MockText("Apples"));
    list.appendChild(apples);
    list.appendChild(new MockText("\n  "));
    const bread = new MockElement('DIV');
    bread.appendChild(new MockText("Bread"));
    list.appendChild(bread);
    documentMock.body.appendChild(list);
    documentMock.body.appendChild(new MockElement('SCRIPT'));

    // The browser renders the list still loading: <div>Loading…</div>
    const commands: CommandType[] = [
        { CreateNode: { id: 140, name: 'div' } },
        { CreateText: { id: 141, value: "Loading…" } },
        { InsertBefore: { parent: 3, child: 140, ref_id: null } },
        { InsertBefore: { parent: 140, child: 141, ref_id: null } },
    ];

    const { report } = hydrate(commands, new MapNodes(), mockedApiLocation());

    assert(report.matched === report.hydratable, "every vnode of the browser found a server node...");
    assert(list.childNodes.length === 1, "...while the rows under them were removed");
    assert(report.discarded === 2, "both rows are counted, neither the whitespace nor the script");
}

// 15. Nodes moved within and between parents, and removed after a move
function testMovesAndRemovals() {
    console.log("\n--- Test hydration 15: Moves and removals ---");
    clearBody();

    const span = (text: string) => {
        const element = new MockElement('SPAN');
        element.appendChild(new MockText(text));
        return element;
    };

    // What the server ends up with after replaying the stream below: A has z, y and B has w.
    const serverA = new MockElement('DIV');
    const serverZ = span("z");
    const serverY = span("y");
    serverA.appendChild(serverZ);
    serverA.appendChild(serverY);
    const serverB = new MockElement('DIV');
    const serverW = span("w");
    serverB.appendChild(serverW);
    documentMock.body.appendChild(serverA);
    documentMock.body.appendChild(serverB);

    const [A, B, x, y, z, w] = [150, 151, 152, 153, 154, 155];
    const commands: CommandType[] = [
        { CreateNode: { id: A, name: 'div' } },
        { CreateNode: { id: B, name: 'div' } },
        { InsertBefore: { parent: 3, child: A, ref_id: null } },
        { InsertBefore: { parent: 3, child: B, ref_id: null } },
    ];
    [x, y, z, w].forEach((id, index) => {
        commands.push(
            { CreateNode: { id, name: 'span' } },
            { CreateText: { id: id + 10, value: "xyzw"[index]! } },
            { InsertBefore: { parent: id, child: id + 10, ref_id: null } },
        );
    });
    // A copy left behind by a move would be the first child of B: it would take the span that
    // is w's, and leave w without one.
    commands.push(
        { InsertBefore: { parent: A, child: x, ref_id: null } },
        { InsertBefore: { parent: A, child: y, ref_id: null } },
        { InsertBefore: { parent: A, child: z, ref_id: null } },  // A: x y z
        { InsertBefore: { parent: B, child: y, ref_id: null } },  // A: x z, B: y
        { InsertBefore: { parent: B, child: w, ref_id: null } },  // B: y w
        { InsertBefore: { parent: A, child: z, ref_id: x } },     // A: z x
        { RemoveNode: { id: x } },                                // A: z
        { InsertBefore: { parent: A, child: y, ref_id: null } },  // A: z y, B: w
    );

    const mapNodes = new MapNodes();
    const { report } = hydrate(commands, mapNodes, mockedApiLocation());

    assert(report.matched === report.hydratable && report.hydratable === 8, "every node is where the server has it");
    assert(report.discarded === 0, "nothing the server rendered is left over");
    assert(mapNodes.getAnyOption(z) as any === serverZ, "z moved before x");
    assert(mapNodes.getAnyOption(y) as any === serverY, "y moved to B and back, and is only in A");
    assert(mapNodes.getAnyOption(w) as any === serverW, "w has its span, so y is not in B anymore");
}

// 16. Inserting before a sibling, wherever the sibling is
function testInsertBeforeAnySibling() {
    console.log("\n--- Test hydration 16: Insert before the last, the first and a middle child ---");
    clearBody();

    // The server's list, in the order the inserts below make: a b c d e
    const server = new MockElement('UL');
    const serverItems = ["a", "b", "c", "d", "e"].map((letter) => {
        const item = new MockElement('LI');
        item.appendChild(new MockText(letter));
        server.appendChild(item);
        return item;
    });
    documentMock.body.appendChild(server);

    const list = 160;
    const [a, b, c, d, e] = [161, 162, 163, 164, 165];
    const commands: CommandType[] = [
        { CreateNode: { id: list, name: 'ul' } },
        { InsertBefore: { parent: 3, child: list, ref_id: null } },
    ];
    [a, b, c, d, e].forEach((id, index) => {
        commands.push(
            { CreateNode: { id, name: 'li' } },
            { CreateText: { id: id + 10, value: "abcde"[index]! } },
            { InsertBefore: { parent: id, child: id + 10, ref_id: null } },
        );
    });
    commands.push(
        { InsertBefore: { parent: list, child: e, ref_id: null } },  // e
        { InsertBefore: { parent: list, child: b, ref_id: e } },     // b e - before the last
        { InsertBefore: { parent: list, child: a, ref_id: b } },     // a b e - before the first
        { InsertBefore: { parent: list, child: d, ref_id: e } },     // a b d e - before the last
        { InsertBefore: { parent: list, child: c, ref_id: d } },     // a b c d e - before a middle one
    );

    const mapNodes = new MapNodes();
    const { report } = hydrate(commands, mapNodes, mockedApiLocation());

    assert(report.matched === report.hydratable && report.discarded === 0, "the list is in the server's order");
    [a, b, c, d, e].forEach((id, index) => {
        assert(mapNodes.getAnyOption(id) as any === serverItems[index], `${"abcde"[index]} is item ${index + 1}`);
    });
}

/// The server's `<div>`, indented as its HTML is, holding a span per text.
function serverSpans(texts: string[]): { div: MockElement, spans: MockElement[] } {
    const div = new MockElement('DIV');
    const spans = texts.map((text) => {
        const span = new MockElement('SPAN');
        span.appendChild(new MockText(text));
        div.appendChild(new MockText("\n  "));
        div.appendChild(span);
        return span;
    });
    div.appendChild(new MockText("\n"));
    documentMock.body.appendChild(div);
    return { div, spans };
}

/// The commands that create a span holding `text`.
function spanCommands(id: number, text: string): CommandType[] {
    return [
        { CreateNode: { id, name: 'span' } },
        { CreateText: { id: id + 1, value: text } },
        { InsertBefore: { parent: id, child: id + 1, ref_id: null } },
    ];
}

// 17. Marker comments are put in without moving what the server sent
function testMarkersArePlacedWithoutMoves() {
    console.log("\n--- Test hydration 17: Markers placed among adopted nodes ---");
    clearBody();

    // The server leaves the markers out: <div><span>a</span><span>b</span></div>
    const server = serverSpans(["a", "b"]);

    // Two `render_value`s: each marker is appended, then its content goes in front of it.
    const [div, m1, a, m2, b] = [170, 171, 172, 175, 176];
    const commands: CommandType[] = [
        { CreateNode: { id: div, name: 'div' } },
        { InsertBefore: { parent: 3, child: div, ref_id: null } },
        { CreateComment: { id: m1, value: "value" } },
        { InsertBefore: { parent: div, child: m1, ref_id: null } },
        ...spanCommands(a, "a"),
        { InsertBefore: { parent: div, child: a, ref_id: m1 } },
        { CreateComment: { id: m2, value: "value" } },
        { InsertBefore: { parent: div, child: m2, ref_id: null } },
        ...spanCommands(b, "b"),
        { InsertBefore: { parent: div, child: b, ref_id: m2 } },
    ];

    const mapNodes = new MapNodes();
    const { report, place } = hydrate(commands, mapNodes, mockedApiLocation());

    // What the batch then creates, as the command replay would: the markers.
    const markers = [m1, m2].map((id) => {
        const marker = new MockComment("value");
        mapNodes.set(id, marker as any);
        return marker;
    });

    MockNode.moved = [];
    place();

    assert(report.matched === report.hydratable, "the spans are adopted");
    const [spanA, spanB] = server.spans;
    const [marker1, marker2] = markers;
    const children = server.div.childNodes;
    assert(children.length === 4 && children[0] === spanA && children[1] === marker1
        && children[2] === spanB && children[3] === marker2, "each marker is right after its span");
    assert(MockNode.moved.length === 0, "no node the server sent was moved");
}

// 18. A node the batch has and the server didn't send goes in among the adopted ones
function testNewElementIsPlacedWithoutMoves() {
    console.log("\n--- Test hydration 18: New element placed among adopted nodes ---");
    clearBody();

    const server = serverSpans(["a", "c"]);

    const [div, a, b, c] = [180, 181, 183, 185];
    const commands: CommandType[] = [
        { CreateNode: { id: div, name: 'div' } },
        { InsertBefore: { parent: 3, child: div, ref_id: null } },
        ...spanCommands(a, "a"),
        { CreateNode: { id: b, name: 'em' } },
        ...spanCommands(c, "c"),
        { InsertBefore: { parent: div, child: a, ref_id: null } },
        { InsertBefore: { parent: div, child: b, ref_id: null } },
        { InsertBefore: { parent: div, child: c, ref_id: null } },
    ];

    const mapNodes = new MapNodes();
    const { place } = hydrate(commands, mapNodes, mockedApiLocation());

    // The <em> has no node from the server, so the replay creates it.
    const em = new MockElement('EM');
    mapNodes.set(b, em as any);

    MockNode.moved = [];
    place();

    const [spanA, spanC] = server.spans;
    const children = server.div.childNodes;
    assert(children.length === 3 && children[0] === spanA && children[1] === em && children[2] === spanC,
        "the <em> is between the spans");
    assert(MockNode.moved.length === 0, "neither span was moved");
}

// 19. A node the batch doesn't put in a parent stays where it is
function testNodesOutsideTheBatchStay() {
    console.log("\n--- Test hydration 19: Whitespace between <head> and <body> ---");
    clearBody();

    // <html> keeps the server's whitespace between <head> and <body>; hydration doesn't go there.
    const html = documentMock.documentElement;
    const whitespace = new MockText("\n");
    html.appendChild(documentMock.head);
    html.appendChild(whitespace);
    html.appendChild(documentMock.body);
    documentMock.body.appendChild(new MockElement('DIV'));

    const commands: CommandType[] = [
        { InsertBefore: { parent: 1, child: 2, ref_id: null } },
        { InsertBefore: { parent: 1, child: 3, ref_id: null } },
        { CreateNode: { id: 190, name: 'div' } },
        { InsertBefore: { parent: 3, child: 190, ref_id: null } },
    ];

    const { place } = hydrate(commands, new MapNodes(), mockedApiLocation());

    MockNode.moved = [];
    place();

    assert(html.childNodes[0] === documentMock.head && html.childNodes[1] === whitespace
        && html.childNodes[2] === documentMock.body, "<html> is as it was");
    assert(MockNode.moved.length === 0, "neither <head> nor <body> was moved");

    for (const node of [...html.childNodes]) {
        node.remove();
    }
}

// 20. The client-side styles go to the end of <head> once, not after every batch
function testStylesStayAtTheEndOfHead() {
    console.log("\n--- Test hydration 20: Styles stay at the end of <head> ---");
    const head = documentMock.head;
    const meta = new MockElement('META');
    head.appendChild(meta);

    // `bulk_update` places the styles after every batch.
    const mapNodes = new MapNodes();
    mapNodes.addStyles();
    const style = head.lastChild;
    assert(head.childNodes.length === 2 && style !== meta, "the first batch puts the styles last");

    MockNode.moved = [];
    mapNodes.addStyles();
    mapNodes.addStyles();
    assert(MockNode.moved.length === 0, "the next batches leave them where they are");

    // Something the page put after them: the styles go back to the end.
    const link = new MockElement('LINK');
    head.appendChild(link);
    mapNodes.addStyles();
    assert(head.lastChild === style && head.childNodes[1] === link, "they are last again");

    for (const node of [...head.childNodes]) {
        node.remove();
    }
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
testEmptyTextIsNotCountedAgainstTheScore();
testEmptyTextDoesNotEatItsSiblings();
testEmptyTextInsideMergedTexts();
testDiscardedServerNodesAreCounted();
testMovesAndRemovals();
testInsertBeforeAnySibling();
testMarkersArePlacedWithoutMoves();
testNewElementIsPlacedWithoutMoves();
testNodesOutsideTheBatchStay();
testStylesStayAtTheEndOfHead();
