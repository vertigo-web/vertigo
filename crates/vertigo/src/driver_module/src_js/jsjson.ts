import { BufferCursor } from "./buffer_cursor";

const JsJsonConst = {
    True: 1,
    False: 2,
    Null: 3,
    Undefined: 4,
    String: 5,
    Number: 6,
    List: 7,
    Object: 8,
    Vec: 9,
} as const;

export type JsJsonType = boolean | null | undefined | string | number | Uint8Array | Array<JsJsonType> | { [key: string]: JsJsonType };

export const jsJsonGetSize = (value: JsJsonType): number => {
    if (value === true || value === false || value === null || value === undefined) {
        return 1;
    }

    if (typeof value === 'string') {
        return 1 + 4 + new TextEncoder().encode(value).length;
    }

    if (typeof value === 'number') {
        return 1 + 8;
    }

    if (value instanceof Uint8Array) {
        return 1 + 4 + value.length;
    }

    if (Array.isArray(value)) {
        let sum = 1 + 4;
        for (const item of value) {
            sum += jsJsonGetSize(item);
        }
        return sum;
    }

    if (typeof value === 'object' && value !== null) {
        let sum = 1 + 2;
        for (const [key, propertyValue] of Object.entries(value)) {
            sum += 4 + new TextEncoder().encode(key).length;
            sum += jsJsonGetSize(propertyValue);
        }
        return sum;
    }

    throw new Error(`jsJsonGetSize: Unknown type ${typeof value}`);
};

export const jsJsonDecodeItem = (buffer: BufferCursor): JsJsonType => {
    const typeId = buffer.getByte();

    if (typeId === JsJsonConst.True) {
        return true;
    }

    if (typeId === JsJsonConst.False) {
        return false;
    }

    if (typeId === JsJsonConst.Null) {
        return null;
    }

    if (typeId === JsJsonConst.Undefined) {
        return undefined;
    }

    if (typeId === JsJsonConst.String) {
        return buffer.getString();
    }

    if (typeId === JsJsonConst.Number) {
        return buffer.getF64();
    }

    if (typeId === JsJsonConst.List) {
        const count = buffer.getU32();
        const list: Array<JsJsonType> = [];

        for (let i = 0; i < count; i++) {
            list.push(jsJsonDecodeItem(buffer));
        }

        return list;
    }

    if (typeId === JsJsonConst.Object) {
        const count = buffer.getU16();
        const obj: { [key: string]: JsJsonType } = {};

        for (let i = 0; i < count; i++) {
            const key = buffer.getString();
            const value = jsJsonDecodeItem(buffer);
            obj[key] = value;
        }

        return obj;
    }

    if (typeId === JsJsonConst.Vec) {
        return buffer.getBuffer();
    }

    throw new Error(`jsJsonDecodeItem: Unknown type id ${typeId}`);
};

export const saveJsJsonToBufferItem = (value: JsJsonType, buffer: BufferCursor): void => {
    if (value === true) {
        buffer.setByte(JsJsonConst.True);
        return;
    }

    if (value === false) {
        buffer.setByte(JsJsonConst.False);
        return;
    }

    if (value === null) {
        buffer.setByte(JsJsonConst.Null);
        return;
    }

    if (value === undefined) {
        buffer.setByte(JsJsonConst.Undefined);
        return;
    }

    if (typeof value === 'string') {
        buffer.setByte(JsJsonConst.String);
        buffer.setString(value);
        return;
    }

    if (typeof value === 'number') {
        buffer.setByte(JsJsonConst.Number);
        buffer.setF64(value);
        return;
    }

    if (value instanceof Uint8Array) {
        buffer.setByte(JsJsonConst.Vec);
        buffer.setBuffer(value);
        return;
    }

    if (Array.isArray(value)) {
        buffer.setByte(JsJsonConst.List);
        buffer.setU32(value.length);

        for (const item of value) {
            saveJsJsonToBufferItem(item, buffer);
        }

        return;
    }

    if (typeof value === 'object' && value !== null) {
        const entries = Object.entries(value);

        buffer.setByte(JsJsonConst.Object);
        buffer.setU16(entries.length);

        for (const [key, propertyValue] of entries) {
            buffer.setString(key);
            saveJsJsonToBufferItem(propertyValue, buffer);
        }

        return;
    }

    throw new Error(`saveJsJsonToBufferItem: Unknown type ${typeof value}`);
};
