import { ApiBrowser } from "./api_browser";
import { convertFromJsValue, convertToJsValue, Guard, JsValueType } from "./arguments";
import { MapNodes } from "./api_browser/dom/map_nodes";

export class JsNode {
    private api: ApiBrowser;
    private nodes: MapNodes<bigint, Element | Comment>;
    private texts: MapNodes<bigint, Text>;
    private wsk: unknown;

    public constructor(
        api: ApiBrowser,
        nodes: MapNodes<bigint, Element | Comment>,
        texts: MapNodes<bigint, Text>,
        wsk: unknown
    ) {
        this.api = api;
        this.nodes = nodes;
        this.texts = texts;
        this.wsk = wsk;
    }

    getByProperty(path: Array<JsValueType>, property: string): JsNode | null {
        try {
            //@ts-expect-error
            const nextCurrentPointer = this.wsk[property];
            return new JsNode(this.api, this.nodes, this.texts, nextCurrentPointer);
        } catch (error) {
            console.error('A problem with get', {
                path,
                property,
                error
            });
            return null;
        }
    }

    public toValue(): JsValueType {
        return convertToJsValue(this.wsk);
    }

    public next(path: Array<JsValueType>, command: JsValueType): JsNode | null {
        if (Array.isArray(command)) {
            const [commandName, ...args] = command;

            if (commandName === 'api') {
                return this.nextApi(path, args);
            }

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

            if (commandName === 'get_props') {
                return this.nextGetProps(path, args);
            }

            console.error('JsNode.next - wrong commandName', commandName);
            return null;
        }

        console.error('JsNode.next - array was expected', { path, command });
        return null;
    }

    nextApi(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        if (args.length === 0) {
            return new JsNode(this.api, this.nodes, this.texts, this.api);
        }

        console.error('nextApi: wrong parameter', {path, args});
        return null;
    }

    nextRoot(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const [firstName, ...rest] = args;

        if (Guard.isString(firstName) && rest.length === 0) {
            if (firstName === 'window') {
                return new JsNode(this.api, this.nodes, this.texts, window);
            }

            if (firstName === 'document') {
                return new JsNode(this.api, this.nodes, this.texts, document);
            }

            console.error(`JsNode.nextRoot: Global name not found -> ${firstName}`, {path, args});
            return null;
        }
        
        if (Guard.isNumber(firstName) && rest.length === 0) {
            const domId = firstName.value;

            const node = this.nodes.getItem(BigInt(domId));

            if (node !== undefined) {
                return new JsNode(this.api, this.nodes, this.texts, node);
            }

            const text = this.texts.getItem(BigInt(domId));

            if (text !== undefined) {
                return new JsNode(this.api, this.nodes, this.texts, text);
            }

            console.error(`JsNode.nextRoot: No node with id=${domId}`, {path, args});
            return null;
        }

        console.error('JsNode.nextRoot: wrong parameter', {path, args});
        return null;
    }

    nextGet(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const [property, ...getArgs] = args;

        if (Guard.isString(property) && getArgs.length === 0) {
            return this.getByProperty(path, property);
        }

        console.error('JsNode.nextGet - wrong parameters', { path, args });
        return null;
    }

    nextSet(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const [property, value, ...setArgs] = args;

        if (Guard.isString(property) && setArgs.length === 0) {
            try {
                //@ts-expect-error
                this.wsk[property] = convertFromJsValue(value);
                return new JsNode(this.api, this.nodes, this.texts, undefined);
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

    nextCall(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const [property, ...callArgs] = args;
        
        if (Guard.isString(property)) {
            try {
                let paramsJs = callArgs.map(convertFromJsValue);
                //@ts-expect-error
                const result = this.wsk[property](...paramsJs);
                return new JsNode(this.api, this.nodes, this.texts, result);
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

    nextGetProps(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const result: Record<string, JsValueType> = {};

        for (const property of args) {
            if (Guard.isString(property)) {
                const value = this.getByProperty(path, property);
                if (value === null) {
                    return null;
                }

                result[property] = value.toValue();
            } else {
                console.error('JsNode.nextGetProps - wrong parameters', { path, args, property });
                return null;
            }
        }

        return new JsNode(this.api, this.nodes, this.texts, result);
    }
}
