import { JsJsonType } from '../../../jsjson';

type SnapshotNodeJson =
    | { Element: { name: string, attrs: Array<{ name: string, value: string }>, children: Array<number> } }
    | { Text: { value: string } }
    | { Comment: { value: string } };

interface SnapshotPayload {
    nodes: Array<SnapshotNodeJson>;
    head: number | null;
    body: number | null;
}

export interface SnapshotResult {
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
/// Elements are visited before text/comment nodes to match Rust's expectations: all elements
/// in the tree are indexed first, then all text/comment nodes.
///
/// Takes the root node, not `Document`, for the same reason.
export const buildSnapshot = (root: Node): SnapshotResult => {
    const payload: SnapshotPayload = { nodes: [], head: null, body: null };
    const nodes: Array<Node> = [];

    // First pass: visit all elements and record their structure
    const visitElements = (node: Node, depth: number): number | null => {
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

            // Process element children first
            for (const child of Array.from(node.childNodes)) {
                if (child.nodeType === ELEMENT_NODE) {
                    visitElements(child, depth + 1);
                }
            }

            return index;
        }

        return null;
    };

    // Second pass: visit text/comment nodes and link them to their parents
    const visitNonElements = (node: Node): void => {
        if (node.nodeType === ELEMENT_NODE) {
            const element = node as Element;
            
            if (isInfrastructure(element)) {
                return;
            }

            // Find this element's index from the first pass
            const elementIndex = nodes.indexOf(node);
            if (elementIndex === -1) {
                return;
            }

            const children: number[] = [];

            for (const child of Array.from(node.childNodes)) {
                if (child.nodeType === ELEMENT_NODE) {
                    // Element child - find its index from first pass
                    const childIndex = nodes.indexOf(child);
                    if (childIndex !== -1) {
                        children.push(childIndex);
                    }
                    visitNonElements(child);
                } else if (child.nodeType === TEXT_NODE || child.nodeType === COMMENT_NODE) {
                    // Add text/comment node now
                    const index = payload.nodes.length;
                    const value = (child as Text | Comment).data;
                    
                    payload.nodes.push(
                        child.nodeType === TEXT_NODE ? { Text: { value } } : { Comment: { value } },
                    );
                    nodes.push(child);
                    children.push(index);
                }
            }

            // Update the element's children list
            const entry = payload.nodes[elementIndex];
            if (entry !== undefined && 'Element' in entry) {
                entry.Element.children = children;
            }
        }
    };

    visitElements(root, 0);
    visitNonElements(root);

    return { payload, nodes };
};

export const snapshotToJson = (payload: SnapshotPayload): JsJsonType => payload as unknown as JsJsonType;
