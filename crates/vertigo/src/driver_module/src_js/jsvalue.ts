import { assertNever } from "./assert_never";
import { BufferCursor, getStringSize } from "./buffer_cursor";
import { GuardJsValue } from "./guard";
import { jsJsonDecodeItem, jsJsonGetSize, JsJsonType, saveJsJsonToBufferItem } from "./jsjson";
import { JsValueType, JsValueConst } from "./jsvalue_types";

//https://github.com/unsplash/unsplash-js/pull/174
// export type AnyJson = boolean | number | string | null | JsonArray | JsonMap;
// export interface JsonMap { [key: string]: AnyJson }
// export interface JsonArray extends Array<AnyJson> {}


const jsValueDecodeItem = (cursor: BufferCursor): JsValueType => {
    const typeParam = cursor.getByte();

    if (typeParam === JsValueConst.U32) {
        return {
            type: JsValueConst.U32,
            value: cursor.getU32()
        };
    }

    if (typeParam === JsValueConst.I32) {
        return {
            type: JsValueConst.I32,
            value: cursor.getI32()
        };
    }

    if (typeParam === JsValueConst.U64) {
        return {
            type: JsValueConst.U64,
            value: cursor.getU64()
        };
    }

    if (typeParam === JsValueConst.I64) {
        return {
            type: JsValueConst.I64,
            value: cursor.getI64()
        };
    }

    if (typeParam === JsValueConst.F64) {
        return {
            type: JsValueConst.F64,
            value: cursor.getF64()
        };
    }

    if (typeParam === JsValueConst.True) {
        return true;
    }

    if (typeParam === JsValueConst.False) {
        return false;
    }

    if (typeParam === JsValueConst.Null) {
        return null;
    }

    if (typeParam === JsValueConst.Undefined) {
        return undefined;
    }

    if (typeParam === JsValueConst.Vec) {
        return cursor.getBuffer();
    }

    if (typeParam === JsValueConst.String) {
        return cursor.getString();
    }

    if (typeParam === JsValueConst.List) {
        const out: Array<JsValueType> = [];

        const listSize = cursor.getU32();

        for (let i=0; i<listSize; i++) {
            out.push(jsValueDecodeItem(cursor))
        }

        return out;
    }

    if (typeParam === JsValueConst.Object) {
        const out: Record<string, JsValueType> = {};

        const listSize = cursor.getU16();

        for (let i=0; i<listSize; i++) {
            const key = cursor.getString();
            const value = jsValueDecodeItem(cursor);
            out[key] = value;
        }

        return {
            type: JsValueConst.Object,
            value: out
        };
    }

    if (typeParam === JsValueConst.Json) {
        const json = jsJsonDecodeItem(cursor);

        return {
            type: JsValueConst.Json,
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
        let sum = 1 + 4;

        for (const item of value) {
            sum += getSize(item);
        }

        return sum;
    }

    if (value instanceof Uint8Array) {
        return 1 + 4 + value.length;
    }

    if (value.type === JsValueConst.I32 || value.type === JsValueConst.U32) {
        return 5;   // 1 + 4
    }

    if (value.type === JsValueConst.I64 || value.type === JsValueConst.U64 || value.type == JsValueConst.F64) {
        return 9;   // 1 + 8
    }

    if (value.type === JsValueConst.Object) {
        let sum = 1 + 2;

        for (const [key, propertyValue] of Object.entries(value.value)) {
            sum += 4 + getStringSize(key);
            sum += getSize(propertyValue);
        }

        return sum;
    }

    if (value.type === JsValueConst.Json) {
        return 1 + jsJsonGetSize(value.value);
    }

    return assertNever(value);
};

const saveToBufferItem = (value: JsValueType, cursor: BufferCursor) => {
    if (value === true) {
        cursor.setByte(JsValueConst.True);
        return;
    }

    if (value === false) {
        cursor.setByte(JsValueConst.False);
        return;
    }

    if (value === null) {
        cursor.setByte(JsValueConst.Null);
        return;
    }

    if (value === undefined) {
        cursor.setByte(JsValueConst.Undefined);
        return;
    }

    if (value instanceof Uint8Array) {
        cursor.setByte(JsValueConst.Vec);
        cursor.setBuffer(value);
        return;
    }

    if (GuardJsValue.isString(value)) {
        cursor.setByte(JsValueConst.String);
        cursor.setString(value);
        return;
    }

    if (Array.isArray(value)) {
        cursor.setByte(JsValueConst.List);
        cursor.setU32(value.length);

        for (const item of value) {
            saveToBufferItem(item, cursor);
        }

        return;
    }

    if (value.type === JsValueConst.U32) {
        cursor.setByte(JsValueConst.U32);
        cursor.setU32(value.value);
        return;
    }

    if (value.type === JsValueConst.I32) {
        cursor.setByte(JsValueConst.I32);
        cursor.setI32(value.value);
        return;
    }

    if (value.type === JsValueConst.U64) {
        cursor.setByte(JsValueConst.U64);
        cursor.setU64(value.value);
        return;
    }

    if (value.type === JsValueConst.I64) {
        cursor.setByte(JsValueConst.I64);
        cursor.setI64(value.value);
        return;
    }

    if (value.type === JsValueConst.F64) {
        cursor.setByte(JsValueConst.F64);
        cursor.setF64(value.value);
        return;
    }

    if (value.type === JsValueConst.Object) {
        const list: Array<[string, JsValueType]> = [];

        for (const [key, propertyValue] of Object.entries(value.value)) {
            list.push([key, propertyValue]);
        }

        cursor.setByte(JsValueConst.Object);
        cursor.setU16(list.length);

        for (const [key, propertyValue] of list) {
            cursor.setString(key);
            saveToBufferItem(propertyValue, cursor);
        }
        return;
    }

    if (value.type === JsValueConst.Json) {
        cursor.setByte(JsValueConst.Json);
        saveJsJsonToBufferItem(value.value, cursor);
        return;
    }

    return assertNever(value);
};


//TODO - do skasowania
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

export const saveToBufferLongPtr = (
    getUint8Memory: () => Uint8Array,
    alloc: (size: number) => number,
    value: JsValueType,
): bigint => {
    if (value === undefined) {
        return 0n;
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

    return (BigInt(ptr) << 32n) + BigInt(size);
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

    if (value.type === JsValueConst.U32 || value.type === JsValueConst.I32) {
        return value.value;
    }

    if (value.type === JsValueConst.U64 || value.type === JsValueConst.I64 || value.type === JsValueConst.F64) {
        return value.value;
    }

    if (value.type === JsValueConst.Object) {
        const result: Record<string, unknown> = {};

        for (const [key, propertyValue] of Object.entries(value.value)) {
            result[key] = convertFromJsValue(propertyValue);
        }

        return result;
    }

    if (value.type === JsValueConst.Json) {
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
        if (value === (value | 0)) {
            // is integer
            if (-(2 ** 31) <= value && value < 2 ** 31) {
                return {
                    type: JsValueConst.I32,
                    value
                };
            }

            return {
                type: JsValueConst.I64,
                value: BigInt(value)
            };
        } else {
            // is float
            return {
                type: JsValueConst.F64,
                value: value,
            }
        }
    }

    if (typeof value === 'bigint') {
        return {
            type: JsValueConst.I64,
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
                type: JsValueConst.Json,
                value: json
            };
        } catch (_error) {
        }

        const result: Record<string, JsValueType> = {};

        for (const [propName, propValue] of Object.entries(value)) {
            result[propName] = convertToJsValue(propValue);
        }

        return {
            type: JsValueConst.Object,
            value: result
        };
    }

    if (Array.isArray(value)) {
        try {
            const list = value.map(convertToJsJson);
            return {
                type: JsValueConst.Json,
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
