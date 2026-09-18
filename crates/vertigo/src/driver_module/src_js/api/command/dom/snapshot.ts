type SnapshotNodeJson =
    | { Element: { name: string, attrs: Array<{ name: string, value: string }>, children: Array<number> } }
    | { Text: { value: string } }
    | { Comment: { value: string } };

interface SnapshotPayload {
    nodes: Array<SnapshotNodeJson>;
    head: number | null;
    body: number | null;
}

interface SnapshotResult {
    /// Goes to rust.
    payload: SnapshotPayload;
    /// Stays here: the index in this table is the address rust uses to reference the node in
    /// NodeAdopt and SnapshotRemove commands. Both lists grow together, so positions line up.
    nodes: Array<Node>;
}

const ELEMENT_NODE = 1;
const TEXT_NODE = 3;
const COMMENT_NODE = 8;

/// Whether this node is infrastructure, not application content.
///
/// The metadata div is detached from the document by the `Metadata` constructor, even before
/// wasm boots, so we don't see it here. The loader script must be skipped explicitly.
const isInfrastructure = (node: Element): boolean => node.hasAttribute('data-vertigo-run-wasm');

/// Walks the tree in pre-order and builds a flat list.
///
/// Flat, not nested, because in the same pass we record nodes into a table - and then the
/// index in the list is simultaneously the address under which we'll find the real node in
/// constant time when an adoption command arrives.
///
/// Node type is recognized through `nodeType`, not `instanceof`: same as the real DOM does,
/// and by the way the only thing the test mocks can do.
///
/// Takes the root node, not `Document`, for the same reason.
export const buildSnapshot = (root: Node): SnapshotResult => {
    const payload: SnapshotPayload = { nodes: [], head: null, body: null };
    const nodes: Array<Node> = [];

    const visit = (node: Node, depth: number): number | null => {
        if (node.nodeType === ELEMENT_NODE) {
            const element = node as Element;

            if (isInfrastructure(element)) {
                return null;
            }

            // Rust compares without regard to case, which handles html and svg at once - thanks
            // to that, the SVG_TAGS table doesn't have to make it into wasm.
            const name = element.tagName.toLowerCase();

            const index = payload.nodes.length;
            payload.nodes.push({
                Element: {
                    name,
                    attrs: element.getAttributeNames().map(attribute => ({
                        name: attribute,
                        value: element.getAttribute(attribute) ?? '',
                    })),
                    children: [],
                },
            });
            nodes.push(node);

            // The only two elements whose id rust knows in advance. Recognized by name at the
            // first level, because the document has exactly one `<head>` and one `<body>`, and
            // comparison with `document.head` wouldn't work for mocks.
            if (depth === 1) {
                if (name === 'head') {
                    payload.head = index;
                }
                if (name === 'body') {
                    payload.body = index;
                }
            }

            const children: Array<number> = [];
            for (const child of Array.from(node.childNodes)) {
                const childIndex = visit(child, depth + 1);
                if (childIndex !== null) {
                    children.push(childIndex);
                }
            }

            const entry = payload.nodes[index];
            if (entry !== undefined && 'Element' in entry) {
                entry.Element.children = children;
            }

            return index;
        }

        if (node.nodeType === TEXT_NODE || node.nodeType === COMMENT_NODE) {
            const index = payload.nodes.length;
            const value = (node as Text | Comment).data;

            // Texts of only whitespace too: in `<pre>` and with inline content they are
            // significant, so js cannot safely filter them.
            payload.nodes.push(
                node.nodeType === TEXT_NODE ? { Text: { value } } : { Comment: { value } },
            );
            nodes.push(node);

            return index;
        }

        return null;
    };

    visit(root, 0);

    return { payload, nodes };
};

