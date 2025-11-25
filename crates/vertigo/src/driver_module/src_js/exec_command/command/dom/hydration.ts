import { CommandType } from "./dom";
import { hydrate_link } from "./injects";
import { AppLocation } from "../../location/AppLocation";

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

export const hydrate = (commands: Array<CommandType>, appLocation: AppLocation) => {
    console.info('hydrate ...');
    const virtualNodes = createVirtualNodes(commands);

    let depth = 0;

    // Traverse and Match
    const hydrateNode = (vNodeId: number, realNode: Node) => {
        const vNode = virtualNodes.get(vNodeId);
        if (!vNode) return;

        // console.log(`Hydration ${depth}: Hydrate node`, vNode, realNode);

        // Match children
        const realChildren = Array.from(realNode.childNodes);
        let realIndex = 0;

        for (const childVId of vNode.children) {
            const childVNode = virtualNodes.get(childVId);
            if (!childVNode) continue;

            // Find a matching real node starting from realIndex
            let matchFound = false;
            for (let i = realIndex; i < realChildren.length; i++) {
                const candidate = realChildren[i];
                if (!candidate) continue;

                let isMatch = false;
                if (childVNode.name) {
                    // Element
                    if (candidate.nodeType === Node.ELEMENT_NODE && (candidate as Element).tagName === childVNode.name) {
                        isMatch = true;
                        // Check attributes
                        if (childVNode.attributes) {
                            const element = candidate as Element;
                            for (const [name, value] of childVNode.attributes) {
                                if (element.getAttribute(name) !== value) {
                                    // console.info(`Hydration ${depth}: Reseting attribute`, element.getAttribute(name), " !== ", value);
                                    element.setAttribute(name, value);
                                }
                            }
                        }
                    }
                } else if (childVNode.value !== undefined) {
                    // Text
                    if (candidate.nodeType === Node.TEXT_NODE) {
                        // For text nodes, we might want to be lenient or exact.
                        // Let's assume exact match or at least non-empty.
                        // Often text nodes might have whitespace differences.
                        // For now, let's just check if it's a text node.
                        // Checking content might be safer.
                        if (candidate.textContent?.replace('\n', ' ').trim() !== childVNode.value?.replace('\n', ' ').trim()) {
                            // console.debug(`Hydration ${depth}: Joint text`, childVNode, candidate);
                            candidate.textContent = childVNode.value || "";
                        }
                        isMatch = true;
                    } else {
                        console.error(`Hydration ${depth}: Text node mismatch`, childVNode, candidate);
                    }
                }

                if (isMatch) {
                    // Remove skipped nodes (realIndex to i)
                    for (let j = realIndex; j < i; j++) {
                        const nodeToRemove = realChildren[j];
                        if (nodeToRemove) {
                            if (nodeToRemove.nodeType !== Node.TEXT_NODE) {
                                console.warn(`Hydration ${depth}: Removing node`, nodeToRemove);
                            }
                            nodeToRemove.remove();
                        }
                    }

                    // Claim it
                    if (candidate instanceof Element || candidate instanceof Comment || candidate instanceof Text) {
                        // Run injects
                        if (candidate instanceof Element) {
                            if (candidate.tagName.toLocaleLowerCase() === 'a') {
                                hydrate_link(candidate, appLocation);
                            }
                        }
                    }

                    // Recurse
                    if (childVNode.name) {
                        depth++;
                        hydrateNode(childVId, candidate);
                        depth--;
                    }

                    // Advance realIndex to i + 1 (consume this node)
                    realIndex = i + 1;
                    matchFound = true;
                    break;
                }
            }

            if (!matchFound) {
                // If we couldn't find a match for this virtual child,
                // we stop trying to match subsequent children in this parent
                // to avoid misalignment. The remaining virtual children will be created.

                // For debug purposes:
                // console.warn(`Hydration ${depth}: No match for vNode`, childVNode, "in parent", vNode);
            }
        }

        // Remove remaining real nodes
        for (let j = realIndex; j < realChildren.length; j++) {
            const nodeToRemove = realChildren[j];
            if (nodeToRemove) {
                if (depth > 0 && nodeToRemove.nodeType !== Node.TEXT_NODE) {
                    console.warn(`Hydration ${depth}: removing node (2)`, nodeToRemove);
                }
                nodeToRemove.remove();
            }
        }
    };

    // Start hydration from Body (id=3) and Head (id=2) if needed
    // Usually we care about Body.
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
