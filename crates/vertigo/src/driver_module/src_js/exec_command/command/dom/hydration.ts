import { CommandType } from "./dom";
import { injects } from "./injects";
import { MapNodes } from "./map_nodes";

interface VirtualNode {
    id: number;
    name?: string;
    value?: string;
    attributes?: Map<string, string>;
    children: Array<number>;
}

import { AppLocation } from "../../location/AppLocation";

export const hydrate = (commands: Array<CommandType>, nodes: MapNodes, appLocation: AppLocation) => {
    const engine = new HydrationEngine(commands, nodes, appLocation);
    engine.hydrate();
};

class HydrationEngine {
    private nodes: MapNodes;
    private appLocation: AppLocation;
    private depth: number = -1;
    private virtualNodes: Map<number, VirtualNode>;

    constructor(commands: Array<CommandType>, nodes: MapNodes, appLocation: AppLocation) {
        this.nodes = nodes;
        this.appLocation = appLocation;
        this.virtualNodes = this.createVirtualNodes(commands);
    }

    public hydrate() {
        // Start hydration from Body (id=3) and Head (id=2) if needed
        // Usually we care about Body.
        const bodyVNode = this.virtualNodes.get(3);
        if (bodyVNode) {
            this.hydrateNode(3, document.body);
        }

        const headVNode = this.virtualNodes.get(2);
        if (headVNode) {
            this.hydrateNode(2, document.head);
        }

        console.log("Hydration complete");
    };

    // Traverse and Match
    private hydrateNode(vNodeId: number, realNode: Node) {
        const vNode = this.virtualNodes.get(vNodeId);
        if (!vNode) return;

        // console.log(`Hydration ${depth}: Hydrate node`, vNode, realNode);

        // Match children
        const realChildren = Array.from(realNode.childNodes);
        let realIndex = 0;
        this.depth++;

        for (const childVId of vNode.children) {
            const childVNode = this.virtualNodes.get(childVId);
            if (!childVNode) continue;

            // Find a matching real node starting from realIndex
            for (let i = realIndex; i < realChildren.length; i++) {
                const candidate = realChildren[i];
                if (!candidate) continue;

                let isMatch = false;
                if (childVNode.name) {
                    // Element
                    isMatch = this.checkElementMatch(candidate, childVNode);
                } else if (childVNode.value !== undefined) {
                    // Text
                    if (candidate.nodeType === Node.TEXT_NODE) {
                        this.checkTextMatch(candidate, childVNode);
                        isMatch = true;
                    } else {
                        console.error(`Hydration ${this.depth}: Text node mismatch`, childVNode, candidate);
                    }
                }

                if (isMatch) {
                    this.removeSkippedNodes(realChildren, realIndex, i);
                    this.claimNode(candidate, childVId);

                    // Recurse if element
                    if (childVNode.name) {
                        this.hydrateNode(childVId, candidate);
                    }

                    // Advance realIndex to i + 1 (consume this node)
                    realIndex = i + 1;
                    break;
                }
            }
        }

        // Remove remaining real nodes
        this.removeSkippedNodes(realChildren, realIndex, realChildren.length);
        this.depth--;
    };

    private checkElementMatch(candidate: Node, childVNode: VirtualNode) {
        let isMatch = false;
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
        return isMatch;
    };

    private checkTextMatch(candidate: Node, childVNode: VirtualNode) {
        // For text nodes, we might want to be lenient or exact.
        // Let's assume exact match or at least non-empty.
        // Often text nodes might have whitespace differences.
        // For now, let's just check if it's a text node.
        // Checking content might be safer.
        if (candidate.textContent?.replace('\n', ' ').trim() !== childVNode.value?.replace('\n', ' ').trim()) {
            // console.debug(`Hydration ${depth}: Joint text`, childVNode, candidate);
            candidate.textContent = childVNode.value || "";
        }
    };

    // Claim node and run injects
    private claimNode(candidate: Node, childVId: number) {
        if (candidate instanceof Element || candidate instanceof Comment || candidate instanceof Text) {
            this.nodes.claimNode(childVId, candidate);

            // Run injects
            if (candidate instanceof Element) {
                injects(candidate, this.appLocation);
            }
        }
    }

    // Remove nodes skipped during matching
    private removeSkippedNodes(realChildren: ChildNode[], realIndex: number, i: number) {
        for (let j = realIndex; j < i; j++) {
            const nodeToRemove = realChildren[j];
            if (nodeToRemove) {
                if (this.depth > 0 && nodeToRemove.nodeType !== Node.TEXT_NODE) {
                    console.warn(`Hydration ${this.depth}: Removing node`, nodeToRemove);
                }
                nodeToRemove.remove();
            }
        }
    }

    private createVirtualNodes(commands: Array<CommandType>): Map<number, VirtualNode> {
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
}
