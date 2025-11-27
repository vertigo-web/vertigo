import { CommandType } from "./dom";
import { hydrate_link } from "./injects";
import { AppLocation } from "../../location/AppLocation";
import { MapNodes } from "./map_nodes";

interface VirtualNode {
    id: number;
    name?: string;
    value?: string;
    attributes?: Map<string, string>;
    children: Array<number>;
}

const createVirtualNodes = (commands: Array<CommandType>): Map<number, VirtualNode> => {
    const virtualNodes = new Map<number, VirtualNode>();
    // Helper to get or create a virtual node
    const getVNode = (id: number): VirtualNode => {
        let node = virtualNodes.get(id);
        if (!node) {
            node = { id, children: [] };
            virtualNodes.set(id, node);
        }
        return node;
    };

    // Build Virtual Tree from Commands
    for (const command of commands) {
        if ('CreateNode' in command) {
            const node = getVNode(command.CreateNode.id);
            node.name = command.CreateNode.name.toUpperCase();
        } else if ('CreateText' in command) {
            const node = getVNode(command.CreateText.id);
            node.value = command.CreateText.value;
        } else if ('InsertBefore' in command) {
            const parent = getVNode(command.InsertBefore.parent);
            const childId = command.InsertBefore.child;
            const refId = command.InsertBefore.ref_id;

            if (refId === null || refId === undefined) {
                parent.children.push(childId);
            } else {
                const index = parent.children.indexOf(refId);
                if (index !== -1) {
                    parent.children.splice(index, 0, childId);
                } else {
                    console.warn(`Hydration: ref_id ${refId} not found in parent ${command.InsertBefore.parent}`);
                    parent.children.push(childId);
                }
            }
        } else if ('SetAttr' in command) {
            const node = getVNode(command.SetAttr.id);
            if (!node.attributes) {
                node.attributes = new Map();
            }
            node.attributes.set(command.SetAttr.name, command.SetAttr.value);
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
 * Checks if an element node matches a virtual node by tag name
 */
const isElementMatch = (candidate: Node, vNode: VirtualNode): boolean => {
    return candidate.nodeType === Node.ELEMENT_NODE &&
        (candidate as Element).tagName === vNode.name;
};

/**
 * Checks if a text node matches a virtual node
 */
const isTextNodeMatch = (candidate: Node, vNode: VirtualNode): boolean => {
    return candidate.nodeType === Node.TEXT_NODE && vNode.value !== undefined;
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
    if (childVNode.name) {
        // Element node matching
        if (isElementMatch(candidate, childVNode)) {
            if (childVNode.attributes) {
                syncAttributes(candidate as Element, childVNode.attributes);
            }
            return true;
        }
    } else if (childVNode.value !== undefined) {
        // Text node matching
        if (isTextNodeMatch(candidate, childVNode)) {
            syncTextContent(candidate, childVNode.value);
            return true;
        } else {
            console.error(`Hydration ${depth}: Text node mismatch`, childVNode, candidate);
        }
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
            if (childVNode.name) {
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
    vNode: VirtualNode,
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
