import { CommandType } from "./dom";
import { hydrate_link } from "./injects";
import { AppLocation } from "../../location/AppLocation";
import { MapNodes } from "./map_nodes";

/**
 * Virtual Element Node - represents a DOM element
 */
interface VirtualElement {
    readonly kind: 'element';
    readonly id: number;
    readonly name: string;
    readonly attributes: Map<string, string>;
    readonly children: ReadonlyArray<number>;
}

/**
 * Virtual Text Node - represents a text node
 * Text nodes cannot have children in the DOM
 */
interface VirtualText {
    readonly kind: 'text';
    readonly id: number;
    readonly value: string;
}

/**
 * Virtual Comment Node - represents a comment node
 * Comment nodes cannot have children in the DOM
 */
interface VirtualComment {
    readonly kind: 'comment';
    readonly id: number;
    readonly value: string;
}

/**
 * Algebraic Data Type for Virtual Nodes
 * A virtual node can be one of: Element, Text, or Comment
 */
type VirtualNode = VirtualElement | VirtualText | VirtualComment;

const createVirtualNodes = (commands: Array<CommandType>): Map<number, VirtualNode> => {
    const virtualNodes = new Map<number, VirtualNode>();

    // First pass: Create nodes
    for (const command of commands) {
        if ('CreateNode' in command) {
            const element: VirtualElement = {
                kind: 'element',
                id: command.CreateNode.id,
                name: command.CreateNode.name.toUpperCase(),
                attributes: new Map(),
                children: []
            };
            virtualNodes.set(command.CreateNode.id, element);
        } else if ('CreateText' in command) {
            const text: VirtualText = {
                kind: 'text',
                id: command.CreateText.id,
                value: command.CreateText.value
            };
            virtualNodes.set(command.CreateText.id, text);
        }
    }

    // Second pass: Set attributes and build tree structure
    for (const command of commands) {
        if ('SetAttr' in command) {
            const node = virtualNodes.get(command.SetAttr.id);
            if (node && node.kind === 'element') {
                node.attributes.set(command.SetAttr.name, command.SetAttr.value);
            } else {
                console.error(`Hydration: SetAttr failed - node ${command.SetAttr.id} is not an element or does not exist`, command.SetAttr);
            }
        } else if ('InsertBefore' in command) {
            const parent = virtualNodes.get(command.InsertBefore.parent);
            if (!parent || parent.kind !== 'element') {
                console.error(`Hydration: InsertBefore failed - parent ${command.InsertBefore.parent} is not an element or does not exist`, command.InsertBefore);
                continue;
            }

            const childId = command.InsertBefore.child;
            const refId = command.InsertBefore.ref_id;

            // Need to cast to mutable array for manipulation
            const children = parent.children as Array<number>;

            if (refId === null || refId === undefined) {
                children.push(childId);
            } else {
                const index = children.indexOf(refId);
                if (index !== -1) {
                    children.splice(index, 0, childId);
                } else {
                    console.warn(`Hydration: ref_id ${refId} not found in parent ${command.InsertBefore.parent}`);
                    children.push(childId);
                }
            }
        }
    }

    return virtualNodes;
};

/**
 * Normalizes text content by replacing newlines with spaces and trimming
 */
const normalizeText = (text: string | null | undefined): string => {
    return text?.replace('\n', ' ').trim() || '';
};

/**
 * Checks if an element node matches a virtual element node by tag name
 */
const isElementMatch = (candidate: Node, vNode: VirtualElement): boolean => {
    return candidate.nodeType === Node.ELEMENT_NODE &&
        (candidate as Element).tagName === vNode.name;
};

/**
 * Checks if a text node matches a virtual text node
 */
const isTextNodeMatch = (candidate: Node, _vNode: VirtualText): boolean => {
    return candidate.nodeType === Node.TEXT_NODE;
};

/**
 * Synchronizes attributes from virtual node to real element
 */
const syncAttributes = (element: Element, attributes: Map<string, string>): void => {
    for (const [name, value] of attributes) {
        if (element.getAttribute(name) !== value) {
            element.setAttribute(name, value);
        }
    }
};

/**
 * Updates text content if it differs from the virtual node value
 */
const syncTextContent = (textNode: Node, expectedValue: string): void => {
    const currentText = normalizeText(textNode.textContent);
    const expectedText = normalizeText(expectedValue);

    if (currentText !== expectedText) {
        textNode.textContent = expectedValue;
    }
};

/**
 * Removes all nodes from the provided array
 */
const removeNodes = (
    nodesToRemove: Array<Node>,
    depth: number
): void => {
    for (const node of nodesToRemove) {
        if (node && node.parentNode) {
            if (node.nodeType !== Node.TEXT_NODE) {
                console.warn(`Hydration ${depth}: Removing node`, node);
            }
            node.parentNode.removeChild(node);
        }
    }
};

/**
 * Removes remaining unmatched nodes from the DOM
 */
const removeRemainingNodes = (
    nodesToRemove: Array<Node>,
    depth: number
): void => {
    for (const node of nodesToRemove) {
        if (node && node.parentNode) {
            if (depth > 0 && node.nodeType !== Node.TEXT_NODE) {
                console.warn(`Hydration ${depth}: removing node (2)`, node);
            }
            node.parentNode.removeChild(node);
        }
    }
};

/**
 * Claims a node and runs necessary injections
 */
const claimAndInjectNode = (
    candidate: Node,
    childVId: number,
    nodes: MapNodes,
    appLocation: AppLocation
): void => {
    if (candidate instanceof Element || candidate instanceof Comment || candidate instanceof Text) {
        nodes.claimNode(childVId, candidate);

        // Run injects for specific element types
        if (candidate instanceof Element) {
            if (candidate.tagName.toLowerCase() === 'a') {
                hydrate_link(candidate, appLocation);
            }
        }
    }
};

/**
 * Attempts to match a virtual node with a real DOM node
 * Returns true if a match was found and processed
 */
const tryMatchNode = (
    childVNode: VirtualNode,
    candidate: Node,
    depth: number
): boolean => {
    switch (childVNode.kind) {
        case 'element':
            // Element node matching
            if (isElementMatch(candidate, childVNode)) {
                syncAttributes(candidate as Element, childVNode.attributes);
                return true;
            }
            break;

        case 'text':
            // Text node matching
            if (isTextNodeMatch(candidate, childVNode)) {
                syncTextContent(candidate, childVNode.value);
                return true;
            }
            console.error(`Hydration ${depth}: Text node mismatch`, childVNode, candidate);
            break;

        case 'comment':
            // Comment node matching - not yet implemented
            break;
    }

    return false;
};

/**
 * Result of attempting to find a match for a virtual node
 */
interface MatchResult {
    matchFound: boolean;
    remainingNodes: Array<Node>;
}

/**
 * Finds and processes a matching real node for a virtual child node
 * Returns the remaining unprocessed nodes
 */
const findAndProcessMatch = (
    childVNode: VirtualNode,
    childVId: number,
    remainingRealNodes: Array<Node>,
    depth: number,
    nodes: MapNodes,
    appLocation: AppLocation,
    hydrateNode: (vNodeId: number, realNode: Node) => void
): MatchResult => {
    for (let i = 0; i < remainingRealNodes.length; i++) {
        const candidate = remainingRealNodes[i];
        if (!candidate) continue;

        const isMatch = tryMatchNode(childVNode, candidate, depth);

        if (isMatch) {
            // Remove skipped nodes before the match
            const skippedNodes = remainingRealNodes.slice(0, i);
            removeNodes(skippedNodes, depth);

            // Claim the matched node
            claimAndInjectNode(candidate, childVId, nodes, appLocation);

            // Recurse for element nodes
            if (childVNode.kind === 'element') {
                hydrateNode(childVId, candidate);
            }

            // Return remaining nodes after the matched one
            return {
                matchFound: true,
                remainingNodes: remainingRealNodes.slice(i + 1)
            };
        }
    }

    return {
        matchFound: false,
        remainingNodes: remainingRealNodes
    };
};

/**
 * Hydrates children of a virtual node against real DOM children
 */
const hydrateChildren = (
    vNode: VirtualElement,
    realChildren: Array<Node>,
    virtualNodes: Map<number, VirtualNode>,
    depth: number,
    nodes: MapNodes,
    appLocation: AppLocation,
    hydrateNode: (vNodeId: number, realNode: Node) => void
): void => {
    let remainingNodes = realChildren;

    for (const childVId of vNode.children) {
        const childVNode = virtualNodes.get(childVId);
        if (!childVNode) continue;

        const result = findAndProcessMatch(
            childVNode,
            childVId,
            remainingNodes,
            depth,
            nodes,
            appLocation,
            hydrateNode
        );

        remainingNodes = result.remainingNodes;

        if (!result.matchFound) {
            // If we couldn't find a match for this virtual child,
            // we stop trying to match subsequent children in this parent
            // to avoid misalignment. The remaining virtual children will be created.
            break;
        }
    }

    // Remove remaining unmatched real nodes
    removeRemainingNodes(remainingNodes, depth);
};

export const hydrate = (commands: Array<CommandType>, nodes: MapNodes, appLocation: AppLocation) => {
    console.info('hydrate ...');
    const virtualNodes = createVirtualNodes(commands);

    let depth = 0;

    // Recursive hydration function
    const hydrateNode = (vNodeId: number, realNode: Node): void => {
        const vNode = virtualNodes.get(vNodeId);
        if (!vNode) return;

        // Only element nodes can have children
        if (vNode.kind !== 'element') return;

        const realChildren = Array.from(realNode.childNodes);

        depth++;
        hydrateChildren(vNode, realChildren, virtualNodes, depth, nodes, appLocation, hydrateNode);
        depth--;
    };

    // Start hydration from Body (id=3) and Head (id=2) if needed
    const bodyVNode = virtualNodes.get(3);
    if (bodyVNode) {
        hydrateNode(3, document.body);
    }

    const headVNode = virtualNodes.get(2);
    if (headVNode) {
        hydrateNode(2, document.head);
    }

    console.log("Hydration complete");
};
