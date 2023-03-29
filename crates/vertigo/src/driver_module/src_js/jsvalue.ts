import { assertNever } from "./assert_never";
import { BufferCursor, getStringSize } from "./buffer_cursor";
import { GuardJsValue } from "./guard";
import { jsJsonDecodeItem, jsJsonGetSize, JsJsonType, saveJsJsonToBufferItem } from "./jsjson";

export type JsValueType
    = { type: 'u32', value: number, }
    | { type: 'i32', value: number, }
    | { type: 'u64', value: bigint, }
    | { type: 'i64', value: bigint, }
    | boolean
    | null
    | undefined
    | string
    | Array<JsValueType>
    | Uint8Array
    | { type: 'object', value: JsValueMapType }
    | { type: 'json', value: JsJsonType };

interface JsValueMapType {
    [key: string]: JsValueType
}

//https://github.com/unsplash/unsplash-js/pull/174
// export type AnyJson = boolean | number | string | null | JsonArray | JsonMap;
// export interface JsonMap { [key: string]: AnyJson }
// export interface JsonArray extends Array<AnyJson> {}


const jsValueDecodeItem = (cursor: BufferCursor): JsValueType => {
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
        const out: Array<JsValueType> = [];

        const listSize = cursor.getU16();

        for (let i=0; i<listSize; i++) {
            out.push(jsValueDecodeItem(cursor))
        }

        return out;
    }

    if (typeParam === 12) {
        const out: Record<string, JsValueType> = {};

        const listSize = cursor.getU16();

        for (let i=0; i<listSize; i++) {
            const key = cursor.getString();
            const value = jsValueDecodeItem(cursor);
            out[key] = value;
        }

        return {
            type:'object',
            value: out
        };
    }

    if (typeParam === 13) {
        const json = jsJsonDecodeItem(cursor);

        return {
            type: 'json',
            value: json
        };
    }

    console.error('typeParam', typeParam);
    throw Error('Invalid branch');
};

export const jsValueDecode = (getUint8Memory: () => Uint8Array, ptr: number, size: number): JsValueType => {
    try {
        const cursor = new BufferCursor(getUint8Memory, ptr, size);
        return jsValueDecodeItem(cursor);
    } catch (err) {
        console.error(err);
        return [];
    }
};

const getSize = (value: JsValueType): number => {
    if (
        value === true ||
        value === false ||
        value === null ||
        value === undefined
    ) {
        return 1;
    }

    if (GuardJsValue.isString(value)) {
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
        return 5;   //1 + 4
    }

    if (value.type === 'i64' || value.type === 'u64') {
        return 9;   //1 + 8
    }

    if (value.type === 'object') {
        let sum = 1 + 2;

        for (const [key, propertyValue] of Object.entries(value.value)) {
            sum += 4 + getStringSize(key);
            sum += getSize(propertyValue);
        }

        return sum;
    }

    if (value.type === 'json') {
        return 1 + jsJsonGetSize(value.value);
    }

    return assertNever(value);
};

const saveToBufferItem = (value: JsValueType, cursor: BufferCursor) => {
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

    if (GuardJsValue.isString(value)) {
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
        const list: Array<[string, JsValueType]> = [];

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

    if (value.type === 'json') {
        cursor.setByte(13);
        saveJsJsonToBufferItem(value.value, cursor);
        return;
    }

    return assertNever(value);
};

export const saveToBuffer = (
    getUint8Memory: () => Uint8Array,
    alloc: (size: number) => number,
    value: JsValueType,
): number => {
    if (value === undefined) {
        return 0;
    }

    const size = getSize(value);
    const ptr = alloc(size);

    const cursor = new BufferCursor(getUint8Memory, ptr, size);
    saveToBufferItem(value, cursor);

    if (size !== cursor.getSavedSize()) {
        console.error({
            size,
            savedSize: cursor.getSavedSize(),
        });

        throw Error('Mismatch between calculated and recorded size');
    }

    return ptr;
};

export const convertFromJsValue = (value: JsValueType): unknown => {
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

    if (GuardJsValue.isString(value)) {
        return value;
    }

    if (Array.isArray(value)) {
        const newList = [];

        for (const item of value) {
            newList.push(convertFromJsValue(item));
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
        const result: Record<string, unknown> = {};

        for (const [key, propertyValue] of Object.entries(value.value)) {
            result[key] = convertFromJsValue(propertyValue);
        }

        return result;
    }

    if (value.type === 'json') {
        return value.value;
    }

    return assertNever(value);
};

//throws an exception when it fails to convert to JsValue
export const convertToJsValue = (value: unknown): JsValueType => {
    if (typeof value === 'string') {
        return value;
    }

    if (value === true || value === false || value === undefined || value === null) {
        return value;
    }

    if (typeof value === 'number') {
        if (-(2**31) <= value && value < 2**31) {
            return {
                type: 'i32',
                value
            };
        }

        return {
            type: 'i64',
            value: BigInt(value)
        };
    }

    if (typeof value === 'bigint') {
        return {
            type: 'i64',
            value
        };
    }

    if (value instanceof Uint8Array) {
        return value;
    }

    if (typeof value === 'object') {
        try {
            const json = convertToJsJson(value);
            return {
                type: 'json',
                value: json
            };
        } catch (_error) {
        }

        const result: Record<string, JsValueType> = {};

        for (const [propName, propValue] of Object.entries(value)) {
            result[propName] = convertToJsValue(propValue);
        }

        return {
            type: 'object',
            value: result
        };
    }

    if (Array.isArray(value)) {
        try {
            const list = value.map(convertToJsJson);
            return {
                type: 'json',
                value: list
            };
        } catch (_error) {
            return value.map(convertToJsValue);
        }
    }

    console.warn('convertToJsValue', value);
    throw Error('It is not possible to convert this data to JsValue');
};

//throws an exception when it fails to convert to JsJson
export const convertToJsJson = (value: unknown): JsJsonType => {
    if (typeof value === 'boolean' || value === null || typeof value === 'number' || typeof value === 'string') {
        return value;
    }

    if (Array.isArray(value)) {
        return value.map(convertToJsJson);
    }

    if (typeof value === 'object') {
        const result: Record<string, JsJsonType> = {};

        for (const [propName, propValue] of Object.entries(value)) {
            result[propName] = convertToJsJson(propValue);
        }

        return result;
    }

    console.warn('convertToJsJson', value);
    throw Error('It is not possible to convert this data to JsJson');
};
