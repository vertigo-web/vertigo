import { AppLocation } from "../../location/AppLocation";
import { CommandType } from "./dom";
import { injects } from "./injects";
import { MapNodes } from "./map_nodes";
import { expectedTagName } from "./tags";

interface VirtualNode {
    id: number;
    name?: string;
    value?: string;
    attributes?: Map<string, string>;
    children: Array<number>;
}

/// What hydration made of the batch it was given. Also parked on
/// `window.__vertigo_hydration`, which is how a browser test can read it: the wasm boots
/// asynchronously, so there is no reliable moment at which a test could install a shim on
/// `console.log` and still catch the line below.
export interface HydrationReport {
    /// False when the batch carried no `<body>` - hydration could not even start.
    rootFound: boolean;
    matched: number;
    /// Vnodes hydration set out to match: the elements and texts under `<head>`/`<body>`.
    hydratable: number;
    /// Marker comments (`render_value` / `render_list` anchors). The server strips comments
    /// from its output, so these have nothing to match and are not counted against the score.
    skipped: number;
    /// Every distinct id the batch mentioned, matchable or not.
    total: number;
}

export const hydrate = (commands: Array<CommandType>, nodes: MapNodes, appLocation: AppLocation): HydrationReport => {
    const engine = new HydrationEngine(commands, nodes, appLocation);
    return engine.hydrate();
};

class HydrationEngine {
    private nodes: MapNodes;
    private appLocation: AppLocation;
    private virtualNodes: Map<number, VirtualNode>;
    private depth: number = -1;
    private matched: number = 0;

    constructor(commands: Array<CommandType>, nodes: MapNodes, appLocation: AppLocation) {
        this.nodes = nodes;
        this.appLocation = appLocation;
        this.virtualNodes = this.createVirtualNodes(commands);
    }

    public hydrate(): HydrationReport {
        // Start hydration from Body (id=3) and Head (id=2) if needed
        // Usually we care about Body.
        const bodyVNode = this.virtualNodes.get(3);
        const headVNode = this.virtualNodes.get(2);

        const report: HydrationReport = {
            rootFound: bodyVNode !== undefined,
            matched: 0,
            ...this.countHydratable(),
            total: this.virtualNodes.size,
        };

        if (!bodyVNode) {
            // Nothing to walk from. The caller will go on to build the tree from scratch and
            // drop the server-rendered markup, so say why rather than reporting a bare 0 %.
            console.error(
                "Hydration skipped: the first DOM batch carries no <body> (node id 3), so the " +
                "server-rendered markup cannot be matched and will be replaced. The app flushed " +
                "DOM changes before the root element was built - see `mount` in exports.rs.",
                report,
            );
            this.publish(report);
            return report;
        }

        this.hydrateNode(3, document.body);

        if (headVNode) {
            this.hydrateNode(2, document.head);
        }

        report.matched = this.matched;

        const percent = report.hydratable === 0
            ? "100.00"
            : (report.matched * 100 / report.hydratable).toFixed(2);

        const summary = `Hydration complete: ${report.matched}/${report.hydratable} matched (${percent}%), ` +
            `${report.skipped} markers skipped, ${report.total} vnodes in batch.`;

        if (report.matched < report.hydratable) {
            console.warn(summary);
        } else {
            console.log(summary);
        }

        this.publish(report);
        return report;
    };

    private publish(report: HydrationReport) {
        try {
            (window as any).__vertigo_hydration = report;
        } catch (_) {
            // Not worth failing a mount over a diagnostic.
        }
    }

    /// Counts what hydration will actually try to match, so the score has a reachable 100 %.
    private countHydratable(): { hydratable: number, skipped: number } {
        let hydratable = 0;
        let skipped = 0;
        const seen = new Set<number>();

        const walk = (id: number, isRoot: boolean) => {
            if (seen.has(id)) {
                return;
            }
            seen.add(id);

            const vNode = this.virtualNodes.get(id);

            // The roots are where the walk starts, not candidates to match.
            if (!isRoot) {
                if (vNode !== undefined && (vNode.name !== undefined || vNode.value !== undefined)) {
                    hydratable++;
                } else {
                    // A marker comment: `InsertBefore` records the child id on its parent, but
                    // only `CreateNode` / `CreateText` give a vnode something to match on, and
                    // the server strips comments from its output anyway. `hydrateNode` skips
                    // these, so they must not count against the score either.
                    skipped++;
                }
            }

            if (!vNode) {
                return;
            }

            for (const childId of vNode.children) {
                walk(childId, false);
            }
        };

        walk(3, true);
        walk(2, true);

        return { hydratable, skipped };
    }

    // Traverse and Match
    private hydrateNode(vNodeId: number, realNode: Node) {
        const vNode = this.virtualNodes.get(vNodeId);
        if (!vNode) return;

        // console.log(`Hydration ${this.depth + 1}: Hydrate node`, vNode, realNode);

        // Match children
        const realChildren = Array.from(realNode.childNodes);
        let realIndex = 0;
        this.depth++;
        let skipTextVNodes = false;

        for (const childVId of vNode.children) {
            const childVNode = this.virtualNodes.get(childVId);
            if (!childVNode) continue;

            // If we are in group of text vnodes, skip them until we find a non-text vnode.
            if (skipTextVNodes && childVNode.value !== undefined) {
                // Deliberately skipped vNodes should be counted as matched
                this.matched++;
                continue;
            } else {
                skipTextVNodes = false;
            }

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
                        // Start skipping eventual group of text vnodes
                        // as they were probably merged into one on SSR side.
                        skipTextVNodes = true;
                    } else {
                        console.error(`Hydration ${this.depth}: Text node mismatch`, childVNode, candidate);
                    }
                }

                if (isMatch) {
                    this.removeSkippedNodes(realChildren, realIndex, i);
                    this.claimNode(candidate, childVId);
                    this.matched++;

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
        const wanted = childVNode.name === undefined ? undefined : expectedTagName(childVNode.name);
        if (candidate.nodeType === Node.ELEMENT_NODE && (candidate as Element).tagName === wanted) {
            isMatch = true;

            const element = candidate as Element;
            const attributes = childVNode.attributes;

            // Adopting a node means taking on its attributes too, in both directions. The
            // server can have rendered attributes this tree does not have - it renders from
            // the same command stream, but a component is free to draw something different
            // under `is_browser()`, and then a leftover `href` or `disabled` would survive on
            // a node that is otherwise the browser's.
            for (const name of element.getAttributeNames()) {
                if (!attributes || !attributes.has(name)) {
                    element.removeAttribute(name);
                }
            }

            if (attributes) {
                for (const [name, value] of attributes) {
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
                if (this.depth !== 0 && nodeToRemove.nodeType !== Node.TEXT_NODE) {
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

        // Unlink a child wherever it currently sits. The server does the same when it replays
        // the stream (`AllElements::insert_before` calls `remove_from_parent`); without it a
        // node that moves between parents shows up under both, and the matcher trips over the
        // copy that is not there.
        const unlink = (childId: number) => {
            for (const node of virtualNodes.values()) {
                const index = node.children.indexOf(childId);
                if (index !== -1) {
                    node.children.splice(index, 1);
                }
            }
        };

        const remove = (id: number) => {
            unlink(id);
            virtualNodes.delete(id);
        };

        // Build Virtual Tree from Commands
        for (const command of commands) {
            if ('CreateNode' in command) {
                const node = getVNode(command.CreateNode.id);
                node.name = command.CreateNode.name;
            } else if ('CreateText' in command) {
                const node = getVNode(command.CreateText.id);
                node.value = command.CreateText.value;
            } else if ('UpdateText' in command) {
                // The text a node ends the batch with is what the server rendered, and so what
                // `checkTextMatch` has to compare against. `CreateText` can carry a stale value:
                // a `Computed` read while a transaction is open returns its cached value, so
                // `DomText::patched` can bake in the old string and correct it right after.
                const node = getVNode(command.UpdateText.id);
                node.value = command.UpdateText.value;
            } else if ('InsertBefore' in command) {
                const parent = getVNode(command.InsertBefore.parent);
                const childId = command.InsertBefore.child;
                const refId = command.InsertBefore.ref_id;

                unlink(childId);

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
            } else if ('RemoveAttr' in command) {
                virtualNodes.get(command.RemoveAttr.id)?.attributes?.delete(command.RemoveAttr.name);
            } else if ('RemoveNode' in command) {
                remove(command.RemoveNode.id);
            } else if ('RemoveText' in command) {
                remove(command.RemoveText.id);
            } else if ('RemoveComment' in command) {
                remove(command.RemoveComment.id);
            }
        }

        return virtualNodes;
    };
}
