import { convertFromJsValue, convertToJsValue, Guard, JsValueType } from "../../arguments";
import { MapNodes } from "./map_nodes";

export class JsNode {
    private wsk: unknown;

    public constructor(wsk: unknown) {
        this.wsk = wsk;
    }

    public static findRoot(
        nodes: MapNodes<bigint, Element | Comment>,
        texts: MapNodes<bigint, Text>,
        args: JsValueType
    ): JsNode | null {
        if (Array.isArray(args)) {
            const [command, firstName, ...rest] = args;

            if (command === 'get' && Guard.isString(firstName) && rest.length === 0) {
                if (firstName === 'window') {
                    return new JsNode(window);
                }

                if (firstName === 'document') {
                    return new JsNode(document);
                }

                console.error(`findRoot: Global name not found -> ${firstName}`);
                return null;
            }
            
            if (command === 'get' && Guard.isNumber(firstName) && rest.length === 0) {
                const domId = firstName.value;

                const node = nodes.getItem(BigInt(domId));

                if (node !== undefined) {
                    return new JsNode(node);
                }

                const text = texts.getItem(BigInt(domId));

                if (text !== undefined) {
                    return new JsNode(text);
                }

                console.error(`findRoot: No node with id=${domId}`);
                return null;
            }
        }

        console.error('findRoot: wrong parameter', args);
        return null;
    }

    getByProperty(path: Array<JsValueType>, property: string): JsNode | null {
        try {
            //@ts-expect-error
            const nextCurrentPointer = this.wsk[property];
            return new JsNode(nextCurrentPointer);
        } catch (error) {
            console.error('A problem with call', {
                path,
                property,
                error
            });
            return null;
        }
    }

    callProperty(path: Array<JsValueType>, property: string, params: Array<JsValueType>): unknown {
        try {
            let paramsJs = params.map(convertFromJsValue);
            //@ts-expect-error
            const result = this.wsk[property](...paramsJs);
            return result;
        } catch (error) {
            console.error('A problem with call', {
                path,
                property,
                error
            });
            return undefined;
        }
    }

    getProperty(path: Array<JsValueType>, property: string): unknown {
        try {
            //@ts-expect-error
            const result = this.wsk[property];
            return result;
        } catch (error) {
            console.error('A problem with get', {
                path,
                property,
                error
            });
            return undefined;
        }
    }

    setProperty(path: Array<JsValueType>, property: string, value: JsValueType) {
        try {
            //@ts-expect-error
            this.wsk[property] = convertFromJsValue(value);
        } catch (error) {
            console.error('A problem with set', {
                path,
                property,
                error
            });
            return undefined;
        }
    }

    public toValue(): JsValueType {
        return convertToJsValue(this.wsk);
    }

    public next(path: Array<JsValueType>, command: JsValueType): JsNode | null {
        if (Array.isArray(command)) {
            const [commandName, ...args] = command;

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
            this.setProperty(path, property, value);
            return new JsNode(undefined);
        }

        console.error('JsNode.nextSet - wrong parameters', { path, args });
        return null;
    }

    nextCall(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const [property, ...callArgs] = args;
        
        if (Guard.isString(property)) {
            const result = this.callProperty(path, property, callArgs);
            return new JsNode(result);
        }

        console.error('JsNode.nextCall - wrong parameters', { path, args });
        return null;
    }

    nextGetProps(path: Array<JsValueType>, args: Array<JsValueType>): JsNode | null {
        const result: Record<string, JsValueType> = {};

        for (const argItem of args) {
            if (Guard.isString(argItem)) {
                const property = argItem;
                const value = this.getByProperty(path, property);
                if (value === null) {
                    return null;
                }

                result[property] = value.toValue();
            } else {
                console.error('JsNode.nextGetProps - wrong parameters', { path, args, argItem });
                return null;
            }
        }

        return new JsNode(result);
    }
}
