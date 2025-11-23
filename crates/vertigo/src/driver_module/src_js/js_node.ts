import { JsJsonType } from "./jsjson";
import { MapNodes } from "./exec_command/command/dom/map_nodes";

export class JsNode {
    private nodes: MapNodes;
    private wsk: unknown;

    public constructor(
        nodes: MapNodes,
        wsk: unknown
    ) {
        this.nodes = nodes;
        this.wsk = wsk;
    }

    getByProperty(path: Array<JsJsonType>, property: string): JsNode | null {
        try {
            //@ts-expect-error
            const nextCurrentPointer = this.wsk[property];
            return new JsNode(this.nodes, nextCurrentPointer);
        } catch (error) {
            console.error('A problem with get', {
                path,
                property,
                error
            });
            return null;
        }
    }

    public toValue(): JsJsonType {
        // Convert JavaScript value to JsJson
        if (this.wsk === null || this.wsk === undefined) {
            return null;
        }
        if (typeof this.wsk === 'boolean') {
            return this.wsk;
        }
        if (typeof this.wsk === 'string') {
            return this.wsk;
        }
        if (typeof this.wsk === 'number') {
            return this.wsk;
        }
        if (Array.isArray(this.wsk)) {
            return this.wsk.map(item => {
                const node = new JsNode(this.nodes, item);
                return node.toValue();
            });
        }
        if (typeof this.wsk === 'object') {
            const result: { [key: string]: JsJsonType } = {};
            for (const [key, value] of Object.entries(this.wsk)) {
                const node = new JsNode(this.nodes, value);
                result[key] = node.toValue();
            }
            return result;
        }
        // Fallback
        return null;
    }

    public next(path: Array<JsJsonType>, command: JsJsonType): JsNode | null {
        if (Array.isArray(command)) {
            const [commandName, ...args] = command;

            if (commandName === 'root') {
                return this.nextRoot(path, args);
            }

            if (commandName === 'get') {
                return this.nextGet(path, args);
            }

            if (commandName === 'set') {
                return this.nextSet(path, args);
            }

            if (commandName === 'call') {
                return this.nextCall(path, args);
            }


            console.error('JsNode.next - wrong commandName', commandName);
            return null;
        }

        console.error('JsNode.next - array was expected', { path, command });
        return null;
    }

    nextRoot(path: Array<JsJsonType>, args: Array<JsJsonType>): JsNode | null {
        const [firstName, ...rest] = args;

        if (typeof firstName === 'string' && rest.length === 0) {
            if (firstName === 'window') {
                return new JsNode(this.nodes, window);
            }

            if (firstName === 'document') {
                return new JsNode(this.nodes, document);
            }

            console.error(`JsNode.nextRoot: Global name not found -> ${firstName}`, { path, args });
            return null;
        }

        if (typeof firstName === 'number' && rest.length === 0) {
            const domId = firstName;

            const node = this.nodes.get_any_option(domId);

            if (node !== undefined) {
                return new JsNode(this.nodes, node);
            }

            console.error(`JsNode.nextRoot: No node with id=${domId}`, { path, args });
            return null;
        }

        console.error('JsNode.nextRoot: wrong parameter', { path, args });
        return null;
    }

    nextGet(path: Array<JsJsonType>, args: Array<JsJsonType>): JsNode | null {
        const [property, ...getArgs] = args;

        if (typeof property === 'string' && getArgs.length === 0) {
            return this.getByProperty(path, property);
        }

        console.error('JsNode.nextGet - wrong parameters', { path, args });
        return null;
    }

    nextSet(path: Array<JsJsonType>, args: Array<JsJsonType>): JsNode | null {
        const [property, value, ...setArgs] = args;

        if (typeof property === 'string' && setArgs.length === 0) {
            try {
                //@ts-expect-error
                this.wsk[property] = value;
                return new JsNode(this.nodes, undefined);
            } catch (error) {
                console.error('A problem with set', {
                    path,
                    property,
                    error
                });
                return null;
            }
        }

        console.error('JsNode.nextSet - wrong parameters', { path, args });
        return null;
    }

    nextCall(path: Array<JsJsonType>, args: Array<JsJsonType>): JsNode | null {
        const [property, ...callArgs] = args;

        if (typeof property === 'string') {
            try {
                //@ts-expect-error
                const result = this.wsk[property](...callArgs);
                return new JsNode(this.nodes, result);
            } catch (error) {
                console.error('A problem with call', {
                    path,
                    property,
                    error
                });
                return null;
            }
        }

        console.error('JsNode.nextCall - wrong parameters', { path, args });
        return null;
    }
}
