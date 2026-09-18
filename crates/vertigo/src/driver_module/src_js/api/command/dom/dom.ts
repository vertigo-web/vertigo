import { AppLocation } from "../../location/AppLocation";
import { CallbackManager } from "./callbackManager";
import { ExportType } from "../../../wasm_module";
import { injects } from "./injects";
import { CommandCursor, Tag, readNames } from "./dom_wire";
import { MapNodes } from "./map_nodes";
import { ModuleControllerType } from "../../../wasm_init";
import { Metadata } from "../../metadata";
import { createElement } from "./tags";
import { buildSnapshot, snapshotToJson } from "./snapshot";
import { JsJsonType } from "../../../jsjson";

export type CommandType = {
    CreateNode: {
        id: number,
        name: string,
    }
} | {
    CreateText: {
        id: number,
        value: string
    }
} | {
    UpdateText: {
        id: number,
        value: string
    }
} | {
    SetAttr: {
        id: number,
        name: string,
        value: string
    }
} | {
    RemoveAttr: {
        id: number,
        name: string
    }
} | {
    RemoveNode: {
        id: number,
    }
} | {
    RemoveText: {
        id: number,
    }
} | {
    InsertBefore: {
        parent: number,
        child: number,
        ref_id: number | null,
    }
} | {
    InsertCss: {
        selector: string | null,
        value: string
    }
} | {
    CreateComment: {
        id: number,
        value: string
    }
} | {
    RemoveComment: {
        id: number,
    }
} | {
    CallbackAdd: {
        id: number,
        event_name: string,
        callback_id: number,
    }
} | {
    CallbackRemove: {
        id: number,
        event_name: string,
        callback_id: number,
    }
} | {
    NodeAdopt: { id: number, snapshot: number }
} | {
    SnapshotRemove: { snapshot: number }
};

const applyFailed = (error: unknown, name: string): void => {
    console.error('bulk_update - item', name, error);
};

/// Position of a name in the batch dictionary, or -1. Compared against the index carried by
/// each command, so the per-command string test it replaces never runs.
const indexOfName = (names: Array<string>, wanted: string): number =>
    names.findIndex((name) => name.toLowerCase() === wanted);

export class DriverDom {
    private appLocation: AppLocation;
    public readonly nodes: MapNodes;
    private readonly callbacks: CallbackManager;
    private snapshotNodes: Array<Node> | null = null;

    public constructor(_metadata: Metadata, appLocation: AppLocation, getWasm: () => ModuleControllerType<ExportType>) {
        this.appLocation = appLocation;
        this.nodes = new MapNodes();
        this.callbacks = new CallbackManager(getWasm);

        document.addEventListener('dragover', (ev): void => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
    }

    /// Response to `DomSnapshotGet`. The node array stays here - rust addresses them by index.
    public snapshot = (): JsJsonType => {
        const { payload, nodes } = buildSnapshot(document.documentElement);
        this.snapshotNodes = nodes;
        return snapshotToJson(payload);
    }

    // `bytes` is the flat command stream - see `dom_wire.ts` and, for the format itself,
    // `crates/vertigo/src/dev/command_wire.rs`. It is a view straight into wasm memory,
    // valid for as long as this call runs, which is why nothing here is deferred.
    public update = (bytes: Uint8Array) => {
        const cursor = new CommandCursor(bytes);
        const names = readNames(cursor);

        // Names arrive as dictionary indices, so the two tests that used to run per command -
        // "is this attribute autofocus" and "is this element an anchor" - are resolved once
        // for the whole batch and then compared as integers. They used to call
        // `toLocaleLowerCase()` on every SetAttr and on every created node.
        const autofocusName = indexOfName(names, 'autofocus');
        const anchorName = indexOfName(names, 'a');

        const setFocus: Set<number> = new Set();

        // Two levels of failure, and they are not the same.
        //
        // Reading a command's fields happens outside the guard: those reads are what advance
        // the cursor, so a throw from one leaves it at an unknown offset with no way to find
        // the next command. That aborts the batch (`decodeFailed`) rather than applying
        // whatever the following bytes happen to look like.
        //
        // Applying a command is guarded per command, exactly as it was before this format
        // existed: one missing node id is logged and the rest of the batch still lands.
        try {
            while (!cursor.isEmpty()) {
                const tag = cursor.byte();

                switch (tag) {
                    case Tag.CreateNode: {
                        const id = cursor.varint();
                        const name = cursor.varint();
                        try {
                            this.createNode(id, names[name] ?? '', name === anchorName);
                        } catch (error) { applyFailed(error, 'CreateNode'); }
                        break;
                    }
                    case Tag.CreateText: {
                        const id = cursor.varint();
                        const value = cursor.string();
                        try { this.createText(id, value); }
                        catch (error) { applyFailed(error, 'CreateText'); }
                        break;
                    }
                    case Tag.UpdateText: {
                        const id = cursor.varint();
                        const value = cursor.string();
                        try { this.updateText(id, value); }
                        catch (error) { applyFailed(error, 'UpdateText'); }
                        break;
                    }
                    case Tag.SetAttr: {
                        const id = cursor.varint();
                        const name = cursor.varint();
                        const value = cursor.string();

                        if (name === autofocusName) {
                            setFocus.add(id);
                        }

                        try { this.setAttr(id, names[name] ?? '', value); }
                        catch (error) { applyFailed(error, 'SetAttr'); }
                        break;
                    }
                    case Tag.RemoveAttr: {
                        const id = cursor.varint();
                        const name = cursor.varint();
                        try { this.removeAttr(id, names[name] ?? ''); }
                        catch (error) { applyFailed(error, 'RemoveAttr'); }
                        break;
                    }
                    case Tag.RemoveNode: {
                        const id = cursor.varint();
                        try { this.removeNode(id); }
                        catch (error) { applyFailed(error, 'RemoveNode'); }
                        break;
                    }
                    case Tag.RemoveText: {
                        const id = cursor.varint();
                        try { this.removeText(id); }
                        catch (error) { applyFailed(error, 'RemoveText'); }
                        break;
                    }
                    case Tag.InsertBefore: {
                        const parent = cursor.varint();
                        const child = cursor.varint();
                        const ref = cursor.optionalId();
                        try { this.nodes.insertBefore(parent, child, ref); }
                        catch (error) { applyFailed(error, 'InsertBefore'); }
                        break;
                    }
                    case Tag.InsertCss: {
                        const selector = cursor.byte() === 0 ? null : cursor.string();
                        const value = cursor.string();
                        try { this.nodes.insertCss(selector, value); }
                        catch (error) { applyFailed(error, 'InsertCss'); }
                        break;
                    }
                    case Tag.CreateComment: {
                        const id = cursor.varint();
                        const value = cursor.string();
                        try { this.nodes.set(id, document.createComment(value)); }
                        catch (error) { applyFailed(error, 'CreateComment'); }
                        break;
                    }
                    case Tag.RemoveComment: {
                        const id = cursor.varint();
                        try { this.nodes.delete("remove_comment", id).remove(); }
                        catch (error) { applyFailed(error, 'RemoveComment'); }
                        break;
                    }
                    case Tag.CallbackAdd: {
                        const id = cursor.varint();
                        const eventName = cursor.string();
                        const callbackId = cursor.varint();
                        try { this.callbacks.add(this.nodes, id, eventName, callbackId); }
                        catch (error) { applyFailed(error, 'CallbackAdd'); }
                        break;
                    }
                    case Tag.CallbackRemove: {
                        const id = cursor.varint();
                        const eventName = cursor.string();
                        const callbackId = cursor.varint();
                        try { this.callbacks.remove(this.nodes, id, eventName, callbackId); }
                        catch (error) { applyFailed(error, 'CallbackRemove'); }
                        break;
                    }
                    case Tag.NodeAdopt: {
                        const id = cursor.varint();
                        const snapshot = cursor.varint();
                        const node = this.snapshotNodes?.[snapshot];

                        if (node === undefined) {
                            console.error(`NodeAdopt: no snapshot node at ${snapshot}`);
                            break;
                        }

                        this.nodes.set(id, node as Element | Comment | Text);

                        if (node.nodeType === 1) {
                            // Without this, capturing clicks in links stops working -
                            // `claimNode` used to do it in the hydration branch.
                            injects(node as Element, this.appLocation);
                        }
                        break;
                    }
                    case Tag.SnapshotRemove: {
                        const snapshot = cursor.varint();
                        const node = this.snapshotNodes?.[snapshot];

                        if (node !== undefined) {
                            (node as ChildNode).remove();
                        }
                        break;
                    }
                    default:
                        throw new Error(`bulk_update: unknown command tag ${tag}`);
                }
            }
        } catch (error) {
            console.error(
                `bulk_update - stream is unreadable at ${cursor.where()}, dropping the rest of the batch`,
                error
            );
        }

        if (setFocus.size > 0) {
            setTimeout(() => {
                for (const id of setFocus) {
                    const node = this.nodes.getNodeElement(`set focus ${id}`, id);
                    node.focus();
                }
            }, 0);
        }

        // The hydration batch is the only one that addresses snapshot nodes.
        this.snapshotNodes = null;

        // Make sure that the client-side generated styles are always the last element of the head
        this.nodes.addStyles();
    }

    private createNode(id: number, name: string, isAnchor: boolean) {
        // Root nodes (html/head/body) already exist in the real DOM
        if (id === 1 || id === 2 || id === 3) {
            return;
        }

        const node = createElement(name);
        this.nodes.set(id, node);

        if (isAnchor) {
            injects(node, this.appLocation);
        }
    }

    private setAttr(id: number, name: string, value: string) {
        const node = this.nodes.getNode("set_attribute", id);
        node.setAttribute(name, value);

        if (name == "value") {
            if (node instanceof HTMLInputElement) {
                node.value = value;
                return;
            }

            if (node instanceof HTMLTextAreaElement) {
                node.value = value;
                node.defaultValue = value;
                return;
            }
        }
    }

    private removeAttr(id: number, name: string) {
        const node = this.nodes.getNode("remove_attribute", id);
        node.removeAttribute(name);

        if (name == "value") {
            if (node instanceof HTMLInputElement) {
                node.value = "";
                return;
            }

            if (node instanceof HTMLTextAreaElement) {
                node.value = "";
                node.defaultValue = "";
                return;
            }
        }
    }

    private removeNode(id: number) {
        // Never remove real document roots
        if (id === 1 || id === 2 || id === 3) {
            return;
        }

        const node = this.nodes.delete("remove_node", id);
        node.remove();
    }

    private createText(id: number, value: string) {
        const text = document.createTextNode(value);
        this.nodes.set(id, text);
    }

    private removeText(id: number) {
        const text = this.nodes.delete("remove_node", id);
        text.remove();
    }

    private updateText(id: number, value: string) {
        const text = this.nodes.getText("set_attribute", id);
        text.textContent = value;
    }

}
