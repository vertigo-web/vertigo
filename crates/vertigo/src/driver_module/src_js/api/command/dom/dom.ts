import { AppLocation } from "../../location/AppLocation";
import { CallbackManager } from "./callbackManager";
import { ExportType } from "../../../wasm_module";
import { hydrate } from "./hydration";
import { injects } from "./injects";
import { MapNodes } from "./map_nodes";
import { ModuleControllerType } from "../../../wasm_init";
import { Metadata } from "../../metadata";

const createElement = (name: string): Element => {
    if (name == "path" || name == "svg") {
        return document.createElementNS("http://www.w3.org/2000/svg", name);
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

const assertNeverCommand = (data: never): never => {
    console.error(data);
    throw Error('unknown command');
};

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

    public update = (commands: Array<CommandType>) => {
        if (this.nodes.hasInitNodes() && this.metadata.getEnabledHydration()) {
            hydrate(commands, this.nodes, this.appLocation);
        }

        const setFocus: Set<number> = new Set();

        for (const command of commands) {
            try {
                this.runCommand(command);
            } catch (error) {
                console.error('bulk_update - item', error, command);
            }

            if ('SetAttr' in command && command.SetAttr.name.toLocaleLowerCase() === 'autofocus') {
                setFocus.add(command.SetAttr.id);
            }
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
    }

    private createNode(id: number, name: string) {
        if (this.nodes.has(id)) {
            return;
        }

        const node = createElement(name);
        this.nodes.set(id, node);

        injects(node, this.appLocation);
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

    private runCommand(command: CommandType) {
        if ('RemoveNode' in command) {
            this.removeNode(command.RemoveNode.id);
            return;
        }

        if ('InsertBefore' in command) {
            this.nodes.insertBefore(command.InsertBefore.parent, command.InsertBefore.child, command.InsertBefore.ref_id === null ? null : command.InsertBefore.ref_id);
            return;
        }

        if ('CreateNode' in command) {
            this.createNode(command.CreateNode.id, command.CreateNode.name);
            return;
        }

        if ('CreateText' in command) {
            this.createText(command.CreateText.id, command.CreateText.value);
            return;
        }

        if ('UpdateText' in command) {
            this.updateText(command.UpdateText.id, command.UpdateText.value);
            return;
        }

        if ('SetAttr' in command) {
            this.setAttr(command.SetAttr.id, command.SetAttr.name, command.SetAttr.value);
            return;
        }

        if ('RemoveAttr' in command) {
            this.removeAttr(command.RemoveAttr.id, command.RemoveAttr.name);
            return;
        }

        if ('RemoveText' in command) {
            this.removeText(command.RemoveText.id);
            return;
        }

        if ('InsertCss' in command) {
            this.nodes.insertCss(command.InsertCss.selector, command.InsertCss.value);
            return;
        }

        if ('CreateComment' in command) {
            const comment = document.createComment(command.CreateComment.value);
            this.nodes.set(command.CreateComment.id, comment);
            return;
        }

        if ('RemoveComment' in command) {
            const comment = this.nodes.delete("remove_comment", command.RemoveComment.id);
            comment.remove();
            return;
        }

        if ('CallbackAdd' in command) {
            this.callbacks.add(this.nodes, command.CallbackAdd.id, command.CallbackAdd.event_name, BigInt(command.CallbackAdd.callback_id));
            return;
        }

        if ('CallbackRemove' in command) {
            this.callbacks.remove(this.nodes, command.CallbackRemove.id, command.CallbackRemove.event_name, BigInt(command.CallbackRemove.callback_id));
            return;
        }

        return assertNeverCommand(command);
    }
}
