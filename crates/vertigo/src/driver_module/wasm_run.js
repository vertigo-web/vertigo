class Cookies {
    get = (cname) => {
        for (const cookie of document.cookie.split(';')) {
            if (cookie === "")
                continue;
            const cookieChunk = cookie.trim().split('=');
            if (cookieChunk.length !== 2) {
                console.warn(`Cookies.get: Incorrect number of cookieChunk => ${cookieChunk.length} in ${cookie}`);
                continue;
            }
            const cookieName = cookieChunk[0];
            const cookieValue = cookieChunk[1];
            if (cookieName === undefined || cookieValue === undefined) {
                console.warn(`Cookies.get: Broken cookie part => ${cookie}`);
                continue;
            }
            if (cookieName === cname) {
                return decodeURIComponent(cookieValue);
            }
        }
        return '';
    };
    set = (cname, cvalue, expires_in) => {
        const cvalueEncoded = cvalue == null ? "" : encodeURIComponent(cvalue);
        const d = new Date();
        d.setTime(d.getTime() + (Number(expires_in) * 1000));
        let expires = "expires=" + d.toUTCString();
        document.cookie = `${cname}=${cvalueEncoded};${expires};path=/;samesite=strict"`;
    };
}

///https://javascript.info/arraybuffer-binary-arrays#dataview
const decoder = new TextDecoder("utf-8");
const encoder = new TextEncoder();
class BufferCursor {
    getUint8Memory;
    ptr;
    size;
    dataView;
    pointer = 0;
    constructor(getUint8Memory, ptr, size) {
        this.getUint8Memory = getUint8Memory;
        this.ptr = ptr;
        this.size = size;
        this.getUint8Memory()[3] = 56;
        this.dataView = new DataView(this.getUint8Memory().buffer, this.ptr, this.size);
    }
    getByte() {
        const value = this.dataView.getUint8(this.pointer);
        this.pointer += 1;
        return value;
    }
    setByte(byte) {
        this.dataView.setUint8(this.pointer, byte);
        this.pointer += 1;
    }
    getU16() {
        const value = this.dataView.getUint16(this.pointer);
        this.pointer += 2;
        return value;
    }
    setU16(value) {
        this.dataView.setUint16(this.pointer, value);
        this.pointer += 2;
    }
    getU32() {
        const value = this.dataView.getUint32(this.pointer);
        this.pointer += 4;
        return value;
    }
    setU32(value) {
        this.dataView.setUint32(this.pointer, value);
        this.pointer += 4;
    }
    getI32() {
        const value = this.dataView.getInt32(this.pointer);
        this.pointer += 4;
        return value;
    }
    setI32(value) {
        this.dataView.setInt32(this.pointer, value);
        this.pointer += 4;
    }
    getU64() {
        const value = this.dataView.getBigUint64(this.pointer);
        this.pointer += 8;
        return value;
    }
    setU64(value) {
        this.dataView.setBigUint64(this.pointer, value);
        this.pointer += 8;
    }
    getI64() {
        const value = this.dataView.getBigInt64(this.pointer);
        this.pointer += 8;
        return value;
    }
    setI64(value) {
        this.dataView.setBigInt64(this.pointer, value);
        this.pointer += 8;
    }
    getBuffer() {
        const size = this.getU32();
        const result = this
            .getUint8Memory()
            .subarray(this.ptr + this.pointer, this.ptr + this.pointer + size);
        this.pointer += size;
        return result;
    }
    setBuffer(buffer) {
        const size = buffer.length;
        this.setU32(size);
        const subbugger = this
            .getUint8Memory()
            .subarray(this.ptr + this.pointer, this.ptr + this.pointer + size);
        subbugger.set(buffer);
        this.pointer += size;
    }
    getString() {
        return decoder.decode(this.getBuffer());
    }
    setString(value) {
        const buffer = encoder.encode(value);
        this.setBuffer(buffer);
    }
}
//https://github.com/unsplash/unsplash-js/pull/174
// export type AnyJson = boolean | number | string | null | JsonArray | JsonMap;
// export interface JsonMap { [key: string]: AnyJson }
// export interface JsonArray extends Array<AnyJson> {}
const argumentsDecodeItem = (cursor) => {
    const typeParam = cursor.getByte();
    if (typeParam === 1) {
        return {
            type: 'u32',
            value: cursor.getU32()
        };
    }
    if (typeParam === 2) {
        return {
            type: 'u32',
            value: cursor.getI32()
        };
    }
    if (typeParam === 3) {
        return {
            type: 'u64',
            value: cursor.getU64()
        };
    }
    if (typeParam === 4) {
        return {
            type: 'i64',
            value: cursor.getI64()
        };
    }
    if (typeParam === 5) {
        return true;
    }
    if (typeParam === 6) {
        return false;
    }
    if (typeParam === 7) {
        return null;
    }
    if (typeParam === 8) {
        return undefined;
    }
    if (typeParam === 9) {
        return cursor.getBuffer();
    }
    if (typeParam === 10) {
        return cursor.getString();
    }
    if (typeParam === 11) {
        const out = [];
        const listSize = cursor.getU16();
        for (let i = 0; i < listSize; i++) {
            out.push(argumentsDecodeItem(cursor));
        }
        return out;
    }
    if (typeParam === 12) {
        const out = {};
        const listSize = cursor.getU16();
        for (let i = 0; i < listSize; i++) {
            const key = cursor.getString();
            const value = argumentsDecodeItem(cursor);
            out[key] = value;
        }
        return {
            type: 'object',
            value: out
        };
    }
    console.error('typeParam', typeParam);
    throw Error('Nieprawidłowe odgałęzienie');
};
const argumentsDecode = (getUint8Memory, ptr, size) => {
    try {
        const cursor = new BufferCursor(getUint8Memory, ptr, size);
        return argumentsDecodeItem(cursor);
    }
    catch (err) {
        console.error(err);
        return [];
    }
};
var Guard;
(function (Guard) {
    Guard.isString = (value) => {
        return typeof value === 'string';
    };
    Guard.isStringOrNull = (value) => {
        return value === null || typeof value === 'string';
    };
    Guard.isNumber = (value) => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === 'i32' || value.type === 'u32';
        }
        return false;
    };
    Guard.isBigInt = (value) => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === 'i64' || value.type === 'u64';
        }
        return false;
    };
})(Guard || (Guard = {}));
const assertNever = (_value) => {
    throw Error("assert never");
};
const getStringSize = (value) => {
    return new TextEncoder().encode(value).length;
};
const getSize = (value) => {
    if (value === true ||
        value === false ||
        value === null ||
        value === undefined) {
        return 1;
    }
    if (Guard.isString(value)) {
        return 1 + 4 + getStringSize(value);
    }
    if (Array.isArray(value)) {
        let sum = 1 + 2;
        for (const item of value) {
            sum += getSize(item);
        }
        return sum;
    }
    if (value instanceof Uint8Array) {
        return 1 + 4 + value.length;
    }
    if (value.type === 'i32' || value.type === 'u32') {
        return 5; //1 + 4
    }
    if (value.type === 'i64' || value.type === 'u64') {
        return 9; //1 + 8
    }
    if (value.type === 'object') {
        let sum = 1 + 2;
        for (const [key, propertyValue] of Object.entries(value.value)) {
            sum += getStringSize(key);
            sum += getSize(propertyValue);
        }
        return sum;
    }
    return assertNever();
};
const saveToBufferItem = (value, cursor) => {
    if (value === true) {
        cursor.setByte(5);
        return;
    }
    if (value === false) {
        cursor.setByte(6);
        return;
    }
    if (value === null) {
        cursor.setByte(7);
        return;
    }
    if (value === undefined) {
        cursor.setByte(8);
        return;
    }
    if (value instanceof Uint8Array) {
        cursor.setByte(9);
        cursor.setBuffer(value);
        return;
    }
    if (Guard.isString(value)) {
        cursor.setByte(10);
        cursor.setString(value);
        return;
    }
    if (Array.isArray(value)) {
        cursor.setByte(11);
        cursor.setU16(value.length);
        for (const item of value) {
            saveToBufferItem(item, cursor);
        }
        return;
    }
    if (value.type === 'u32') {
        cursor.setByte(1);
        cursor.setU32(value.value);
        return;
    }
    if (value.type === 'i32') {
        cursor.setByte(2);
        cursor.setI32(value.value);
        return;
    }
    if (value.type === 'u64') {
        cursor.setByte(3);
        cursor.setU64(value.value);
        return;
    }
    if (value.type === 'i64') {
        cursor.setByte(4);
        cursor.setI64(value.value);
        return;
    }
    if (value.type === 'object') {
        const list = [];
        for (const [key, propertyValue] of Object.entries(value.value)) {
            list.push([key, propertyValue]);
        }
        cursor.setByte(12);
        cursor.setU16(list.length);
        for (const [key, propertyValue] of list) {
            cursor.setString(key);
            saveToBufferItem(propertyValue, cursor);
        }
        return;
    }
    return assertNever();
};
const saveToBuffer = (getUint8Memory, alloc, value) => {
    const size = getSize(value);
    const ptr = alloc(size);
    const cursor = new BufferCursor(getUint8Memory, ptr, size);
    saveToBufferItem(value, cursor);
    return ptr;
};
class JsValueBuilder {
    getUint8Memory;
    alloc;
    params;
    constructor(getUint8Memory, alloc) {
        this.getUint8Memory = getUint8Memory;
        this.alloc = alloc;
        this.params = [];
    }
    push_string(value) {
        this.params.push(value);
    }
    push_buffer(buf) {
        this.params.push(buf);
    }
    push_u32(value) {
        this.params.push({
            type: 'u32',
            value
        });
    }
    push_i32(value) {
        this.params.push({
            type: 'i32',
            value
        });
    }
    push_u64(value) {
        this.params.push({
            type: 'u64',
            value
        });
    }
    push_i64(value) {
        this.params.push({
            type: 'i64',
            value
        });
    }
    push_null() {
        this.params.push(null);
    }
    push_bool(value) {
        this.params.push(value);
    }
    push_list(build) {
        const sub_params = new JsValueBuilder(this.getUint8Memory, this.alloc);
        build(sub_params);
        this.params.push(sub_params.params);
    }
    saveToBuffer() {
        return saveToBuffer(this.getUint8Memory, this.alloc, this.params);
    }
    saveListItem(value) {
        return saveToBuffer(this.getUint8Memory, this.alloc, value);
    }
    debug() {
        console.info('debug budowania listy', this.params);
    }
}
const convertFromListItem = (value) => {
    if (value === true) {
        return true;
    }
    if (value === false) {
        return false;
    }
    if (value === null) {
        return null;
    }
    if (value === undefined) {
        return undefined;
    }
    if (value instanceof Uint8Array) {
        return value;
    }
    if (Guard.isString(value)) {
        return value;
    }
    if (Array.isArray(value)) {
        const newList = [];
        for (const item of value) {
            newList.push(convertFromListItem(item));
        }
        return newList;
    }
    if (value.type === 'u32' || value.type === 'i32') {
        return value.value;
    }
    if (value.type === 'u64' || value.type === 'i64') {
        return value.value;
    }
    if (value.type === 'object') {
        const result = {};
        for (const [key, propertyValue] of Object.entries(value.value)) {
            result[key] = convertFromListItem(propertyValue);
        }
        return result;
    }
    return assertNever();
};
const convertToListItem = (value) => {
    if (typeof value === 'string') {
        return value;
    }
    if (value === true || value === false || value === undefined || value === null) {
        return null;
    }
    if (typeof value === 'number') {
        return {
            type: 'i32',
            value
        };
    }
    if (typeof value === 'bigint') {
        return {
            type: 'i64',
            value
        };
    }
    console.error('convertToListItem', value);
    throw Error('TODO');
};

class JsNode {
    wsk;
    constructor(wsk) {
        this.wsk = wsk;
    }
    getByProperty(property) {
        try {
            //@ts-expect-error
            const nextCurrentPointer = this.wsk[property];
            return new JsNode(nextCurrentPointer);
        }
        catch (error) {
            console.error(error);
            return null;
        }
    }
    callProperty(path, property, params) {
        try {
            let paramsJs = params.map(convertFromListItem);
            //@ts-expect-error
            const result = this.wsk[property](...paramsJs);
            return convertToListItem(result);
        }
        catch (error) {
            console.error('A problem with call', {
                path,
                property,
                error
            });
            return undefined;
        }
    }
    getProperty(path, property) {
        try {
            //@ts-expect-error
            const result = this.wsk[property];
            return convertToListItem(result);
        }
        catch (error) {
            console.error('A problem with get', {
                path,
                property,
                error
            });
            return undefined;
        }
    }
    setProperty(path, property, value) {
        try {
            //@ts-expect-error
            this.wsk[property] = convertFromListItem(value);
        }
        catch (error) {
            console.error('A problem with set', {
                path,
                property,
                error
            });
            return undefined;
        }
    }
}
class FindDom {
    nodes;
    texts;
    constructor(nodes, texts) {
        this.nodes = nodes;
        this.texts = texts;
    }
    find_root_dom(firstName) {
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
    findDomByPath(path) {
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
                }
                else {
                    console.error('find_dom_by_path - wrong parameters', path);
                    return null;
                }
            }
        }
        catch (error) {
            console.error(`Error when searching for path=${path}`, error);
            return null;
        }
    }
}

class MapNodes {
    data;
    constructor() {
        this.data = new Map();
    }
    set(key, value) {
        this.data.set(key, value);
    }
    getItem(key) {
        return this.data.get(key);
    }
    mustGetItem(key) {
        const item = this.data.get(key);
        if (item === undefined) {
            throw Error(`item not found=${key}`);
        }
        return item;
    }
    get(label, key, callback) {
        const item = this.data.get(key);
        if (item === undefined) {
            console.error(`${label}->get: Item id not found = ${key}`);
        }
        else {
            callback(item);
        }
    }
    get2(label, key1, key2, callback) {
        const node1 = this.data.get(key1);
        const node2 = this.data.get(key2);
        if (node1 === undefined) {
            console.error(`${label}->get: Item id not found = ${key1}`);
            return;
        }
        if (node2 === undefined) {
            console.error(`${label}->get: Item id not found = ${key2}`);
            return;
        }
        callback(node1, node2);
    }
    delete(label, key, callback) {
        const item = this.data.get(key);
        this.data.delete(key);
        if (item === undefined) {
            console.error(`${label}->delete: Item id not found = ${key}`);
        }
        else {
            this.data.delete(key);
            callback(item);
        }
    }
}

const createElement = (name) => {
    if (name == "path" || name == "svg") {
        return document.createElementNS("http://www.w3.org/2000/svg", name);
    }
    else {
        return document.createElement(name);
    }
};
const assertNeverCommand = (data) => {
    console.error(data);
    throw Error('unknown command');
};
class DriverDom {
    getWasm;
    nodes;
    texts;
    all;
    findDom;
    constructor(getWasm) {
        this.getWasm = getWasm;
        this.nodes = new MapNodes();
        this.texts = new MapNodes();
        this.all = new Map();
        this.findDom = new FindDom(this.nodes, this.texts);
        document.addEventListener('mousedown', (event) => {
            const target = event.target;
            if (target instanceof Element) {
                const id = this.all.get(target);
                if (id !== undefined) {
                    this.getWasm().exports.dom_mousedown(id);
                    return;
                }
            }
            console.warn('mousedown ignore', target);
        }, false);
        // document.addEventListener('mouseover', (event) => {
        //     const target = event.target;
        //     if (target instanceof Element) {
        //         const id = this.all.get(target);
        //         if (id === undefined) {
        //             this.getWasm().exports.dom_mouseover(0n);
        //             return;
        //         }
        //         this.getWasm().exports.dom_mouseover(id);
        //         return;
        //     }
        //     console.warn('mouseover ignore', target);
        // }, false);
        document.addEventListener('keydown', (event) => {
            const target = event.target;
            if (target instanceof Element && event instanceof KeyboardEvent) {
                const id = this.all.get(target);
                const new_params = this.getWasm().newList();
                if (id === undefined) {
                    new_params.push_null();
                }
                else {
                    new_params.push_u64(id);
                }
                new_params.push_string(event.key);
                new_params.push_string(event.code);
                new_params.push_bool(event.altKey);
                new_params.push_bool(event.ctrlKey);
                new_params.push_bool(event.shiftKey);
                new_params.push_bool(event.metaKey);
                const new_params_id = new_params.saveToBuffer();
                const stopPropagate = this.getWasm().exports.dom_keydown(new_params_id);
                if (stopPropagate > 0) {
                    event.preventDefault();
                    event.stopPropagation();
                }
                return;
            }
            console.warn('keydown ignore', target);
        }, false);
        document.addEventListener('input', (event) => {
            const target = event.target;
            if (target instanceof Element) {
                const id = this.all.get(target);
                if (id !== undefined) {
                    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
                        const new_params = this.getWasm().newList();
                        new_params.push_u64(id);
                        new_params.push_string(target.value);
                        const new_params_id = new_params.saveToBuffer();
                        this.getWasm().exports.dom_oninput(new_params_id);
                        return;
                    }
                    console.warn('input ignore', target);
                    return;
                }
            }
            console.warn('input ignore', target);
        }, false);
        document.addEventListener('dragover', (ev) => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
        document.addEventListener('drop', (event) => {
            event.preventDefault();
            const dom_id = this.getIdByTarget(event.target);
            if (dom_id === null) {
                console.warn('drop ignore', event.target);
                return;
            }
            if (event.dataTransfer === null) {
                console.error('dom -> drop -> dataTransfer null');
            }
            else {
                const files = [];
                for (let i = 0; i < event.dataTransfer.items.length; i++) {
                    const item = event.dataTransfer.items[i];
                    if (item === undefined) {
                        console.error('dom -> drop -> item - undefined');
                    }
                    else {
                        const file = item.getAsFile();
                        if (file === null) {
                            console.error(`dom -> drop -> index:${i} -> It's not a file`);
                        }
                        else {
                            files.push(file
                                .arrayBuffer()
                                .then((data) => ({
                                name: file.name,
                                data: new Uint8Array(data),
                            })));
                        }
                    }
                }
                if (files.length) {
                    Promise.all(files).then((files) => {
                        const params = this.getWasm().newList();
                        params.push_u64(dom_id);
                        params.push_list((params_files) => {
                            for (const file of files) {
                                params_files.push_list((params_details) => {
                                    params_details.push_string(file.name);
                                    params_details.push_buffer(file.data);
                                });
                            }
                        });
                        this.getWasm().exports.dom_ondropfile(params.saveToBuffer());
                    });
                }
                else {
                    console.error('No files to send');
                }
            }
        }, false);
    }
    getIdByTarget(target) {
        if (target instanceof Element) {
            const id = this.all.get(target);
            return id ?? null;
        }
        return null;
    }
    mount_node(root_id) {
        this.nodes.get("append_to_body", root_id, (root) => {
            document.body.appendChild(root);
        });
    }
    create_node(id, name) {
        const node = createElement(name);
        this.nodes.set(id, node);
        this.all.set(node, id);
    }
    set_attribute(id, name, value) {
        this.nodes.get("set_attribute", id, (node) => {
            if (node instanceof Element) {
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
            else {
                console.error("set_attribute error");
            }
        });
    }
    remove_node(id) {
        this.nodes.delete("remove_node", id, (node) => {
            this.all.delete(node);
            const parent = node.parentElement;
            if (parent !== null) {
                parent.removeChild(node);
            }
        });
    }
    create_text(id, value) {
        const text = document.createTextNode(value);
        this.texts.set(id, text);
        this.all.set(text, id);
    }
    remove_text(id) {
        this.texts.delete("remove_node", id, (text) => {
            this.all.delete(text);
            const parent = text.parentElement;
            if (parent !== null) {
                parent.removeChild(text);
            }
        });
    }
    update_text(id, value) {
        this.texts.get("set_attribute", id, (text) => {
            text.textContent = value;
        });
    }
    get_node(label, id, callback) {
        const node = this.nodes.getItem(id);
        if (node !== undefined) {
            callback(node);
            return;
        }
        const text = this.texts.getItem(id);
        if (text !== undefined) {
            callback(text);
            return;
        }
        console.error(`${label}->get_node: Item id not found = ${id}`);
        return;
    }
    insert_before(parent, child, ref_id) {
        this.nodes.get("insert_before", parent, (parentNode) => {
            this.get_node("insert_before child", child, (childNode) => {
                if (ref_id === null || ref_id === undefined) {
                    parentNode.insertBefore(childNode, null);
                }
                else {
                    this.get_node('insert_before ref', ref_id, (ref_node) => {
                        parentNode.insertBefore(childNode, ref_node);
                    });
                }
            });
        });
    }
    insert_css(selector, value) {
        const style = document.createElement('style');
        const content = document.createTextNode(`${selector} { ${value} }`);
        style.appendChild(content);
        document.head.appendChild(style);
    }
    dom_bulk_update = (value) => {
        const setFocus = new Set();
        try {
            const commands = JSON.parse(value);
            for (const command of commands) {
                this.bulk_update_command(command);
                if (command.type === 'set_attr' && command.name.toLocaleLowerCase() === 'autofocus') {
                    setFocus.add(command.id);
                }
            }
        }
        catch (error) {
            console.warn('buil_update - check in: https://jsonformatter.curiousconcept.com/');
            console.warn('bulk_update - param', value);
            console.error('bulk_update - incorrectly json data', error);
        }
        if (setFocus.size > 0) {
            setTimeout(() => {
                for (const id of setFocus) {
                    this.nodes.get(`set focus ${id}`, BigInt(id), (node) => {
                        if (node instanceof HTMLElement) {
                            node.focus();
                        }
                        else {
                            console.error('setfocus: HTMLElement expected');
                        }
                    });
                }
            }, 0);
        }
    };
    bulk_update_command(command) {
        if (command.type === 'remove_node') {
            this.remove_node(BigInt(command.id));
            return;
        }
        if (command.type === 'insert_before') {
            this.insert_before(BigInt(command.parent), BigInt(command.child), command.ref_id === null ? null : BigInt(command.ref_id));
            return;
        }
        if (command.type === 'mount_node') {
            this.mount_node(BigInt(command.id));
            return;
        }
        if (command.type === 'create_node') {
            this.create_node(BigInt(command.id), command.name);
            return;
        }
        if (command.type === 'create_text') {
            this.create_text(BigInt(command.id), command.value);
            return;
        }
        if (command.type === 'update_text') {
            this.update_text(BigInt(command.id), command.value);
            return;
        }
        if (command.type === 'set_attr') {
            this.set_attribute(BigInt(command.id), command.name, command.value);
            return;
        }
        if (command.type === 'remove_text') {
            this.remove_text(BigInt(command.id));
            return;
        }
        if (command.type === 'insert_css') {
            this.insert_css(command.selector, command.value);
            return;
        }
        if (command.type === 'create_comment') {
            const comment = document.createComment(command.value);
            this.nodes.set(BigInt(command.id), comment);
            return;
        }
        if (command.type === 'remove_comment') {
            this.nodes.delete("remove_comment", BigInt(command.id), (comment) => {
                const parent = comment.parentElement;
                if (parent !== null) {
                    parent.removeChild(comment);
                }
            });
            return;
        }
        return assertNeverCommand(command);
    }
    dom_call = (path, property, params) => {
        const target = this.findDom.findDomByPath(path);
        if (target === null) {
            return null;
        }
        return target.callProperty(path, property, params);
    };
    dom_get = (path, property) => {
        const target = this.findDom.findDomByPath(path);
        if (target === null) {
            return null;
        }
        return target.getProperty(path, property);
    };
    dom_set = (path, property, value) => {
        const target = this.findDom.findDomByPath(path);
        if (target === null) {
            return;
        }
        target.setProperty(path, property, value);
    };
    /*
        dom-path

        [43, 'focus']                       (HtmlElement.focus)
        ['window', 'location', 'hash']      (window.location.hash)
    */
    dom_access = (_path) => {
        throw Error('TODO');
    };
}

class EventEmmiter {
    events;
    constructor() {
        this.events = new Set();
    }
    on(callback) {
        let isActive = true;
        const onExec = (param) => {
            if (isActive) {
                callback(param);
            }
        };
        this.events.add(onExec);
        return () => {
            isActive = false;
            this.events.delete(onExec);
        };
    }
    trigger(param) {
        const eventsCopy = Array.from(this.events.values());
        for (const itemCallbackToRun of eventsCopy) {
            try {
                itemCallbackToRun(param);
            }
            catch (err) {
                console.error(err);
            }
        }
    }
    get size() {
        return this.events.size;
    }
}

const createPromiseValue = () => {
    let resolve = null;
    let reject = null;
    const promise = new Promise((localResolve, localReject) => {
        resolve = localResolve;
        reject = localReject;
    });
    if (resolve === null) {
        throw Error('createPromiseValue - resolve is null');
    }
    if (reject === null) {
        throw Error('createPromiseValue - reject is null');
    }
    const promiseValue = {
        resolve,
        reject,
    };
    return [promiseValue, promise];
};
class PromiseBoxRace {
    promiseResolveReject = null;
    promise;
    constructor() {
        const [promiseResolveReject, promise] = createPromiseValue();
        this.promiseResolveReject = promiseResolveReject;
        this.promise = promise;
    }
    resolve = (value) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;
        if (promiseResolveReject === null) {
            return;
        }
        promiseResolveReject.resolve(value);
    };
    reject = (err) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;
        if (promiseResolveReject === null) {
            return;
        }
        promiseResolveReject.reject(err);
    };
    isFulfilled = () => {
        return this.promiseResolveReject === null;
    };
}

const timeout = async (timeout) => {
    return new Promise((resolve) => {
        setTimeout(resolve, timeout);
    });
};
const reconnectDelay = async (label, timeout_retry) => {
    console.info(`${label} wait ${timeout_retry}ms`);
    await timeout(timeout_retry);
    console.info(`${label} go forth`);
};
class LogContext {
    host;
    constructor(host) {
        this.host = host;
    }
    formatLog = (message) => `Socket ${this.host} ==> ${message}`;
}
class SocketConnection {
    eventMessage;
    close;
    send;
    constructor(close, send) {
        this.eventMessage = new EventEmmiter();
        this.close = close;
        this.send = send;
    }
    static connect(log, host, timeout) {
        const result = new PromiseBoxRace();
        const done = new PromiseBoxRace();
        const socket = new WebSocket(host);
        let isClose = false;
        console.info(log.formatLog('starting ...'));
        const closeSocket = () => {
            if (isClose) {
                return;
            }
            console.info(log.formatLog('close'));
            isClose = true;
            result.resolve(null);
            done.resolve();
            socket.close();
        };
        const socketConnection = new SocketConnection(closeSocket, (message) => {
            if (isClose) {
                return;
            }
            socket.send(message);
        });
        setTimeout(() => {
            if (result.isFulfilled() === false) {
                console.error(log.formatLog(`timeout (${timeout}ms)`));
                closeSocket();
            }
        }, timeout);
        const onOpen = () => {
            console.info(log.formatLog('open'));
            result.resolve(socketConnection);
        };
        const onError = (error) => {
            console.error(log.formatLog('error'), error);
            closeSocket();
        };
        const onMessage = (event) => {
            if (isClose) {
                return;
            }
            const dataRaw = event.data;
            if (typeof dataRaw === 'string') {
                socketConnection.eventMessage.trigger(dataRaw);
                return;
            }
            console.error(log.formatLog('onMessage - expected string'), dataRaw);
        };
        socket.addEventListener('open', onOpen);
        socket.addEventListener('error', onError);
        socket.addEventListener('close', closeSocket);
        socket.addEventListener('message', onMessage);
        return {
            socket: result.promise,
            done: done.promise
        };
    }
    static startSocket(host, timeout_connection, timeout_retry, onMessage) {
        let isConnect = true;
        let socketConnection = null;
        const log = new LogContext(host);
        (async () => {
            while (isConnect) {
                const openSocketResult = SocketConnection.connect(log, host, timeout_connection);
                const socket = await openSocketResult.socket;
                if (socket === null) {
                    await reconnectDelay(log.formatLog('reconnect after error'), timeout_retry);
                    continue;
                }
                socketConnection = socket;
                onMessage({
                    type: 'socket',
                    socket
                });
                socket.eventMessage.on(message => {
                    onMessage({
                        type: 'message',
                        message
                    });
                });
                await openSocketResult.done;
                onMessage({
                    type: 'close'
                });
                if (!isConnect) {
                    console.info(log.formatLog('disconnect (1)'));
                    return;
                }
                await reconnectDelay(log.formatLog('reconnect after close'), timeout_retry);
            }
            console.info(log.formatLog('disconnect (2)'));
        })().catch((error) => {
            console.error(error);
        });
        return {
            send: (message) => {
                if (socketConnection === null) {
                    console.error('send fail - missing connection', message);
                }
                else {
                    socketConnection.send(message);
                }
            },
            dispose: () => {
                isConnect = false;
                socketConnection?.close();
            }
        };
    }
}

const assertNeverMessage = (data) => {
    console.error(data);
    throw Error('unknown message');
};
class DriverWebsocket {
    getWasm;
    controllerList;
    socket;
    constructor(getWasm) {
        this.getWasm = getWasm;
        this.controllerList = new Map();
        this.socket = new Map();
    }
    websocket_register_callback = (host, callback_id) => {
        const wasm = this.getWasm();
        let controller = SocketConnection.startSocket(host, 5000, //timeout connection 
        3000, //timeout reconnection
        (message) => {
            if (this.controllerList.has(callback_id) === false) {
                return;
            }
            if (message.type === 'socket') {
                this.socket.set(callback_id, message.socket);
                wasm.exports.websocket_callback_socket(callback_id);
                return;
            }
            if (message.type === 'message') {
                const new_params = wasm.newList();
                new_params.push_u32(callback_id);
                new_params.push_string(message.message);
                const new_params_id = new_params.saveToBuffer();
                wasm.exports.websocket_callback_message(new_params_id);
                return;
            }
            if (message.type === 'close') {
                wasm.exports.websocket_callback_close(callback_id);
                this.socket.delete(callback_id);
                return;
            }
            return assertNeverMessage(message);
        });
        this.controllerList.set(callback_id, controller);
    };
    websocket_unregister_callback = (callback_id) => {
        const controller = this.controllerList.get(callback_id);
        if (controller === undefined) {
            console.error('Expected controller');
            return;
        }
        controller.dispose();
        this.controllerList.delete(callback_id);
    };
    websocket_send_message = (callback_id, message) => {
        const socket = this.socket.get(callback_id);
        if (socket === undefined) {
            console.error(`Missing socket connection for callback_id=${callback_id}`);
        }
        else {
            socket.send(message);
        }
    };
}

class Fetch {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    fetch_send_request = (request_id, method, url, headers, body) => {
        const wasm = this.getWasm();
        const headers_record = JSON.parse(headers);
        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) => response.text()
            .then((responseText) => {
            const new_params = this.getWasm().newList();
            new_params.push_u32(request_id); //request_id
            new_params.push_bool(true); //ok
            new_params.push_u32(response.status); //http code
            new_params.push_string(responseText); //body
            let params_id = new_params.saveToBuffer();
            wasm.exports.fetch_callback(params_id);
        })
            .catch((err) => {
            console.error('fetch error (2)', err);
            const responseMessage = new String(err).toString();
            const new_params = this.getWasm().newList();
            new_params.push_u32(request_id); //request_id
            new_params.push_bool(false); //ok
            new_params.push_u32(response.status); //http code
            new_params.push_string(responseMessage); //body
            let params_id = new_params.saveToBuffer();
            wasm.exports.fetch_callback(params_id);
        }))
            .catch((err) => {
            console.error('fetch error (1)', err);
            const responseMessage = new String(err).toString();
            const new_params = this.getWasm().newList();
            new_params.push_u32(request_id); //request_id
            new_params.push_bool(false); //ok
            new_params.push_u32(0); //http code
            new_params.push_string(responseMessage); //body
            let params_id = new_params.saveToBuffer();
            wasm.exports.fetch_callback(params_id);
        });
    };
}

class HashRouter {
    constructor(getWasm) {
        window.addEventListener("hashchange", () => {
            const params = getWasm().newList();
            params.push_string(this.get());
            const ptr = params.saveToBuffer();
            getWasm().exports.hashrouter_hashchange_callback(ptr);
        }, false);
    }
    push = (new_hash) => {
        location.hash = new_hash;
    };
    get() {
        return decodeURIComponent(location.hash.substr(1));
    }
}

const instant_now = () => {
    return Date.now();
};

class Interval {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    interval_set = (duration, callback_id) => {
        const timer_id = setInterval(() => {
            this.getWasm().exports.interval_run_callback(callback_id);
        }, Number(duration));
        return timer_id;
    };
    interval_clear = (timer_id) => {
        clearInterval(timer_id);
    };
    timeout_set = (duration, callback_id) => {
        const timeout_id = setTimeout(() => {
            this.getWasm().exports.timeout_run_callback(callback_id);
        }, duration);
        return timeout_id;
    };
    timeout_clear = (timer_id) => {
        clearTimeout(timer_id);
    };
}

// import { argumentsDecode, ListItemType } from "./arguments";
const fetchModule = async (wasmBinPath, imports) => {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
        try {
            const module = await WebAssembly.instantiateStreaming(fetch(wasmBinPath), imports);
            return module;
        }
        catch (err) {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", err);
        }
    }
    console.info('fetchModule by WebAssembly.instantiate');
    const resp = await fetch(wasmBinPath);
    const binary = await resp.arrayBuffer();
    const module_instance = await WebAssembly.instantiate(binary, imports);
    return module_instance;
};
const wasmInit = async (wasmBinPath, imports) => {
    const module_instance = await fetchModule(wasmBinPath, imports);
    let cachegetUint8Memory = new Uint8Array(1);
    const getUint8Memory = () => {
        if (module_instance.instance.exports.memory instanceof WebAssembly.Memory) {
            if (cachegetUint8Memory.buffer !== module_instance.instance.exports.memory.buffer) {
                console.info('getUint8Memory: reallocate the Uint8Array for a new size', module_instance.instance.exports.memory.buffer.byteLength);
                cachegetUint8Memory = new Uint8Array(module_instance.instance.exports.memory.buffer);
            }
            return cachegetUint8Memory;
        }
        else {
            throw Error('Missing memory');
        }
    };
    //@ts-expect-error
    const exports = module_instance.instance.exports;
    const decodeArguments = (ptr, size) => argumentsDecode(getUint8Memory, ptr, size);
    const newList = () => new JsValueBuilder(getUint8Memory, exports.alloc);
    return {
        exports,
        decodeArguments,
        getUint8Memory,
        newList
    };
};

const initCookie = (getWasm, cookies) => (method, args) => {
    if (method === 'get') {
        const [name, ...rest] = args;
        if (Guard.isString(name) && rest.length === 0) {
            const value = cookies.get(name);
            const params = getWasm().newList();
            params.push_string(value);
            return params.saveToBuffer();
        }
        console.error('js-call -> module -> cookie -> get: incorrect parameters', args);
        return 0;
    }
    if (method === 'set') {
        const [name, value, expires_in, ...rest] = args;
        if (Guard.isString(name) &&
            Guard.isString(value) &&
            Guard.isBigInt(expires_in) &&
            rest.length === 0) {
            cookies.set(name, value, expires_in.value);
            return 0;
        }
        console.error('js-call -> module -> cookie -> set: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> cookie: incorrect parameters', args);
    return 0;
};

const initDom = (dom) => (method, args) => {
    if (method === 'dom_bulk_update') {
        const [value, ...rest] = args;
        if (Guard.isString(value) && rest.length === 0) {
            dom.dom_bulk_update(value);
            return 0;
        }
        console.error('js-call -> module -> dom -> dom_bulk_update: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> dom: incorrect parameters', args);
    return 0;
};

const initFetch = (fetch) => (method, args) => {
    if (method === 'send') {
        const [requestId, httpMethod, url, headers, body, ...rest] = args;
        if (Guard.isNumber(requestId) &&
            Guard.isString(httpMethod) &&
            Guard.isString(url) &&
            Guard.isString(headers) &&
            Guard.isStringOrNull(body) &&
            rest.length === 0) {
            fetch.fetch_send_request(requestId.value, httpMethod, url, headers, body);
            return 0;
        }
        console.error('js-call -> module -> fetch -> send: incorrect parameters', args);
    }
    console.error('js-call -> module -> fetch: incorrect parameters', args);
    return 0;
};

const initHashrouter = (getWasm, hashRouter) => (method, args) => {
    if (method === 'push') {
        const [new_hash, ...rest] = args;
        if (Guard.isString(new_hash) && rest.length === 0) {
            hashRouter.push(new_hash);
            return 0;
        }
        console.error('js-call -> module -> hashrouter -> push: incorrect parameters', args);
        return 0;
    }
    if (method === 'get') {
        if (args.length === 0) {
            const hash = hashRouter.get();
            const params = getWasm().newList();
            params.push_string(hash);
            return params.saveToBuffer();
        }
        console.error('js-call -> module -> hashrouter -> get: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> hashrouter: incorrect parameters', args);
    return 0;
};

const initWebsocketModule = (websocket) => (method, args) => {
    if (method === 'register_callback') {
        const [host, callback_id, ...rest] = args;
        if (Guard.isString(host) &&
            Guard.isNumber(callback_id) &&
            rest.length === 0) {
            websocket.websocket_register_callback(host, callback_id.value);
            return 0;
        }
        console.error('js-call -> module -> websocket -> register_callback: incorrect parameters', args);
        return 0;
    }
    if (method === 'unregister_callback') {
        const [callback_id, ...rest] = args;
        if (Guard.isNumber(callback_id) &&
            rest.length === 0) {
            websocket.websocket_unregister_callback(callback_id.value);
            return 0;
        }
        console.error('js-call -> module -> websocket -> unregister_callback: incorrect parameters', args);
        return 0;
    }
    if (method === 'send_message') {
        const [callback_id, message, ...rest] = args;
        if (Guard.isNumber(callback_id) &&
            Guard.isString(message) &&
            rest.length === 0) {
            websocket.websocket_send_message(callback_id.value, message);
            return 0;
        }
        console.error('js-call -> module -> websocket -> send_message: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> websocket: incorrect parameters', args);
    return 0;
};

const js_call = (decodeArguments, getWasm, fetch, cookies, dom, hashRouter, websocket) => {
    const modules = {
        hashrouter: initHashrouter(getWasm, hashRouter),
        websocket: initWebsocketModule(websocket),
        cookie: initCookie(getWasm, cookies),
        fetch: initFetch(fetch),
        dom: initDom(dom),
    };
    return (ptr, size) => {
        const args = decodeArguments(ptr, size);
        // console.info('js_call', args);        //for debug
        if (Array.isArray(args)) {
            const [modulePrefix, moduleName, newFunction, ...restNew] = args;
            if (modulePrefix === 'module' &&
                Guard.isString(moduleName) &&
                Guard.isString(newFunction)) {
                const toRun = modules[moduleName];
                if (toRun === undefined) {
                    console.error(`js-call: unknown module ${moduleName}`, args);
                    return 0;
                }
                return toRun(newFunction, restNew);
            }
            console.error('js-call: incorrect parameters', args);
            return 0;
        }
        console.error("js-call: List of parameters was expected: ", args);
        return 0;
    };
};

class WasmModule {
    wasm;
    constructor(wasm) {
        this.wasm = wasm;
    }
    start_application() {
        this.wasm.exports.start_application();
    }
    static async create(wasmBinPath) {
        let wasmModule = null;
        const getWasm = () => {
            if (wasmModule === null) {
                throw Error('Wasm is no initialized');
            }
            return wasmModule;
        };
        const cookies = new Cookies();
        const interval = new Interval(getWasm);
        const hashRouter = new HashRouter(getWasm);
        const fetchModule = new Fetch(getWasm);
        const websocket = new DriverWebsocket(getWasm);
        const dom = new DriverDom(getWasm);
        wasmModule = await wasmInit(wasmBinPath, {
            mod: {
                panic_message: (ptr, size) => {
                    const decoder = new TextDecoder("utf-8");
                    const m = getWasm().getUint8Memory().subarray(ptr, ptr + size);
                    const message = decoder.decode(m);
                    console.error('PANIC', message);
                },
                js_call: js_call((ptr, size) => getWasm().decodeArguments(ptr, size), getWasm, fetchModule, cookies, dom, hashRouter, websocket),
                interval_set: interval.interval_set,
                interval_clear: interval.interval_clear,
                timeout_set: interval.timeout_set,
                timeout_clear: interval.timeout_clear,
                instant_now,
                dom_call: (ptr, size) => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const [domPath, property, params, ...rest] = args;
                        if (Array.isArray(domPath) && Guard.isString(property) && Array.isArray(params) && rest.length === 0) {
                            const response = dom.dom_call(domPath, property, params);
                            return getWasm().newList().saveListItem(response);
                        }
                    }
                    console.error('dom_call - wrong parameters', args);
                    return 0;
                },
                dom_get: (ptr, size) => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const [domPath, property, ...rest] = args;
                        if (Array.isArray(domPath) && Guard.isString(property) && rest.length === 0) {
                            const response = dom.dom_get(domPath, property);
                            return getWasm().newList().saveListItem(response);
                        }
                    }
                    console.error('dom_get - wrong parameters', args);
                    return 0;
                },
                dom_set: (ptr, size) => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const [domPath, property, value, ...rest] = args;
                        if (Array.isArray(domPath) && Guard.isString(property) && rest.length === 0) {
                            dom.dom_set(domPath, property, value);
                            return;
                        }
                    }
                    console.error('dom_set - wrong parameters', args);
                }
            }
        });
        return new WasmModule(wasmModule);
    }
}

const runModule = async (wasmBinPath) => {
    console.info(`Wasm module: "${wasmBinPath}" -> start`);
    const wasmModule = await WasmModule.create(wasmBinPath);
    console.info(`Wasm module: "${wasmBinPath}" -> initialized`);
    wasmModule.start_application();
    console.info(`Wasm module: "${wasmBinPath}" -> launched start_application function`);
};

export { runModule };
