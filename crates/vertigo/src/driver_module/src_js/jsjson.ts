import { BufferCursor, getStringSize } from "./buffer_cursor";

export type JsJsonType
    = boolean
    | null
    | number
    | string
    | Array<JsJsonType>
    | JsJsonMapType;

interface JsJsonMapType {
    [key: string]: JsJsonType
}

export const jsJsonGetSize = (value: JsJsonType): number => {

    if (typeof value === 'boolean') {
        return 1;
    }

    if (value === null) {
        return 1;
    }

    if (typeof value === 'string') {
        return 1 + 4 + getStringSize(value);
    }

    if (Array.isArray(value)) {
        let sum = 1 + 4;

        for (const item of value) {
            sum += jsJsonGetSize(item);
        }

        return sum;
    }

    if (typeof value === 'number') {
        return 9;   //1 + 8
    }

    //object
    let sum = 1 + 2;

    for (const [key, propertyValue] of Object.entries(value)) {
        sum += 4 + getStringSize(key);
        sum += jsJsonGetSize(propertyValue);
    }

    return sum;
};

export const jsJsonDecodeItem = (cursor: BufferCursor): JsJsonType => {
    const typeParam = cursor.getByte();

    if (typeParam === 1) {
        return true;
    }

    if (typeParam === 2) {
        return false;
    }

    if (typeParam === 3) {
        return null;
    }

    if (typeParam === 4) {
        return cursor.getString();
    }

    if (typeParam === 5) {
        return cursor.getF64();
    }

    if (typeParam === 6) {
        const out: Array<JsJsonType> = [];

        const listSize = cursor.getU32();

        for (let i=0; i<listSize; i++) {
            out.push(jsJsonDecodeItem(cursor))
        }

        return out;
    }

    //object
    const out: Record<string, JsJsonType> = {};

    const listSize = cursor.getU16();

    for (let i=0; i<listSize; i++) {
        const key = cursor.getString();
        const value = jsJsonDecodeItem(cursor);
        out[key] = value;
    }

    return out;
}

export const saveJsJsonToBufferItem = (value: JsJsonType, cursor: BufferCursor) => {
    if (value === true) {
        cursor.setByte(1);
        return;
    }

    if (value === false) {
        cursor.setByte(2);
        return;
    }

    if (value === null) {
        cursor.setByte(3);
        return;
    }

    if (typeof value === 'string') {
        cursor.setByte(4);
        cursor.setString(value);
        return;
    }

    if (typeof value === 'number') {
        cursor.setByte(5);
        cursor.setF64(value);
        return;
    }

    if (Array.isArray(value)) {
        cursor.setByte(6);
        cursor.setU32(value.length);

        for (const item of value) {
            saveJsJsonToBufferItem(item, cursor);
        }

        return;
    }

    //object
    const list: Array<[string, JsJsonType]> = [];

    for (const [key, propertyValue] of Object.entries(value)) {
        list.push([key, propertyValue]);
    }

    cursor.setByte(7);
    cursor.setU16(list.length);

    for (const [key, propertyValue] of list) {
        cursor.setString(key);
        saveJsJsonToBufferItem(propertyValue, cursor);
    }
};
