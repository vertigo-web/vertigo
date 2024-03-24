import { BufferCursor, getStringSize } from "./buffer_cursor";

const JsJsonConst = {
    True: 1,
    False: 2,
    Null: 3,

    String: 4,
    Number: 5,
    List: 6,
    Object: 7,
} as const;

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

    if (typeParam === JsJsonConst.True) {
        return true;
    }

    if (typeParam === JsJsonConst.False) {
        return false;
    }

    if (typeParam === JsJsonConst.Null) {
        return null;
    }

    if (typeParam === JsJsonConst.String) {
        return cursor.getString();
    }

    if (typeParam === JsJsonConst.Number) {
        return cursor.getF64();
    }

    if (typeParam === JsJsonConst.List) {
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
        cursor.setByte(JsJsonConst.True);
        return;
    }

    if (value === false) {
        cursor.setByte(JsJsonConst.False);
        return;
    }

    if (value === null) {
        cursor.setByte(JsJsonConst.Null);
        return;
    }

    if (typeof value === 'string') {
        cursor.setByte(JsJsonConst.String);
        cursor.setString(value);
        return;
    }

    if (typeof value === 'number') {
        cursor.setByte(JsJsonConst.Number);
        cursor.setF64(value);
        return;
    }

    if (Array.isArray(value)) {
        cursor.setByte(JsJsonConst.List);
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

    cursor.setByte(JsJsonConst.Object);
    cursor.setU16(list.length);

    for (const [key, propertyValue] of list) {
        cursor.setString(key);
        saveJsJsonToBufferItem(propertyValue, cursor);
    }
};
