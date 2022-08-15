import { convertFromListItem, convertToListItem, Guard, JsValueType } from "../../arguments";
import { MapNodes } from "./map_nodes";


class JsNode {
    private wsk: unknown;

    public constructor(wsk: unknown) {
        this.wsk = wsk;
    }

    public getByProperty(property: string): JsNode | null {
        try {
            //@ts-expect-error
            const nextCurrentPointer = this.wsk[property];
            return new JsNode(nextCurrentPointer);
        } catch (error) {
            console.error(error);
            return null;
        }
    }

    public callProperty(path: Array<JsValueType>, property: string, params: Array<JsValueType>): JsValueType {
        try {
            let paramsJs = params.map(convertFromListItem);
            //@ts-expect-error
            const result = this.wsk[property](...paramsJs);
            return convertToListItem(result);
        } catch (error) {
            console.error('A problem with call', {
                path,
                property,
                error
            });
            return undefined;
        }
    }

    public getProperty(path: Array<JsValueType>, property: string): JsValueType {
        try {
            //@ts-expect-error
            const result = this.wsk[property];
            return convertToListItem(result);
        } catch (error) {
            console.error('A problem with get', {
                path,
                property,
                error
            });
            return undefined;
        }
    }

    public setProperty(path: Array<JsValueType>, property: string, value: JsValueType) {
        try {
            //@ts-expect-error
            this.wsk[property] = convertFromListItem(value);
        } catch (error) {
            console.error('A problem with set', {
                path,
                property,
                error
            });
            return undefined;
        }
    }
}


export class FindDom {
    constructor(
        private readonly nodes: MapNodes<bigint, Element | Comment>,
        private readonly texts: MapNodes<bigint, Text>
    ) {
    }

    private find_root_dom(firstName: JsValueType): JsNode | null {
        if (Guard.isString(firstName)) {
            if (firstName === 'window') {
                return new JsNode(window);
            }

            if (firstName === 'document') {
                return new JsNode(document);
            }

            console.error(`Global name not found: ${firstName}`);
            return null;
        }
        
        if (Guard.isNumber(firstName)) {
            const domId = firstName.value;

            const node = this.nodes.getItem(BigInt(domId));

            if (node !== undefined) {
                return new JsNode(node);
            }

            const text = this.texts.getItem(BigInt(domId));

            if (text !== undefined) {
                return new JsNode(text);
            }

            console.error(`No node with id=${domId}`);
            return null;
        }

        console.error('find_root_dom - wrong parameter', firstName);
        return null;
    }

    public findDomByPath(path: Array<JsValueType>): JsNode | null {
        const [firstName, ...restPath] = path;

        try {
            let currentPointer = this.find_root_dom(firstName);

            if (currentPointer === null) {
                return null;
            }

            while (true) {
                const nextProperty = restPath.shift();

                if (nextProperty === undefined) {
                    return currentPointer;
                }

                if (Guard.isString(nextProperty)) {
                    currentPointer = currentPointer.getByProperty(nextProperty);

                    if (currentPointer === null) {
                        return null;
                    }
                } else {
                    console.error('find_dom_by_path - wrong parameters', path);
                    return null;
                }
            }
        } catch (error) {
            console.error(`Error when searching for path=${path}`, error)
            return null;
        }
    }
}

