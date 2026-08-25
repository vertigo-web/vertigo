import { AppLocation } from "../../location/AppLocation";
import { CallbackManager } from "./callbackManager";
import { ExportType } from "../../../wasm_module";
import { hydrate } from "./hydration";
import { hydrateLink } from "./injects";
import { CommandCursor, Tag, decodeCommands, readNames } from "./dom_wire";
import { MapNodes } from "./map_nodes";
import { ModuleControllerType } from "../../../wasm_init";
import { Metadata } from "../../metadata";

// Workaround, remove when https://github.com/vertigo-web/vertigo/issues/539 is done.
const SVG_TAGS = new Set([
    "animate", "animateMotion", "animateTransform", "circle", "clipPath", "defs",
    "desc", "discard", "ellipse", "feBlend", "feColorMatrix", "feComponentTransfer",
    "feComposite", "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap",
    "feDistantLight", "feDropShadow", "feFlood", "feFuncA", "feFuncB", "feFuncG",
    "feFuncR", "feGaussianBlur", "feImage", "feMerge", "feMergeNode", "feMorphology",
    "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight", "feTile",
    "feTurbulence", "filter", "foreignObject", "g", "hatch", "hatchpath", "image",
    "line", "linearGradient", "marker", "mask", "metadata", "mpath", "path", "pattern",
    "polygon", "polyline", "radialGradient", "rect", "set", "stop", "svg", "switch",
    "symbol", "text", "textPath", "tspan", "use", "view",
    "svg:a", "svg:title", "svg:desc", "svg:script", "svg:style"
]);

const createElement = (name: string): Element => {
    if (SVG_TAGS.has(name)) {
        return document.createElementNS("http://www.w3.org/2000/svg", name.replace("svg:", ""));
    } else {
        return document.createElement(name);
    }
}

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

    public constructor(private readonly metadata: Metadata, appLocation: AppLocation, getWasm: () => ModuleControllerType<ExportType>) {
        this.appLocation = appLocation;
        this.nodes = new MapNodes();
        this.callbacks = new CallbackManager(getWasm);

        document.addEventListener('dragover', (ev): void => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
    }

    // `bytes` is the flat command stream - see `dom_wire.ts` and, for the format itself,
    // `crates/vertigo/src/dev/command_wire.rs`. It is a view straight into wasm memory,
    // valid for as long as this call runs, which is why nothing here is deferred.
    public update = (bytes: Uint8Array) => {
        if (this.nodes.hasInitNodes() && this.metadata.getEnabledHydration()) {
            // First flush only, so the object form is worth building here and nowhere else.
            hydrate(decodeCommands(bytes), this.nodes, this.appLocation);
        }

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

        this.nodes.removeInitNodes();

        // Make sure that the client-side generated styles are always the last element of the head
        this.nodes.addStyles();
    }

    private createNode(id: number, name: string, isAnchor: boolean) {
        // Root nodes (html/head/body) already exist in the real DOM
        if (id === 1 || id === 2 || id === 3) {
            return;
        }

        if (this.nodes.has(id)) {
            return;
        }

        const node = createElement(name);
        this.nodes.set(id, node);

        if (isAnchor) {
            hydrateLink(node, this.appLocation);
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
        if (this.nodes.has(id)) {
            return;
        }

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
