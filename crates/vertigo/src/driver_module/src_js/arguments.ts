///https://javascript.info/arraybuffer-binary-arrays#dataview

const decoder = new TextDecoder("utf-8");
const encoder = new TextEncoder();

class BufferCursor {
    private dataView: DataView;
    private pointer: number = 0;

    constructor(
        private getUint8Memory: () => Uint8Array,
        private ptr: number,
        private size: number,
    ) {
        this.getUint8Memory()[3] = 56;
        this.dataView = new DataView(
            this.getUint8Memory().buffer,
            this.ptr,
            this.size
        );
    }

    public getByte(): number {
        const value = this.dataView.getUint8(this.pointer);
        this.pointer += 1;
        return value;
    }

    public setByte(byte: number) {
        this.dataView.setUint8(this.pointer, byte);
        this.pointer += 1;
    }

    public getU16(): number {
        const value = this.dataView.getUint16(this.pointer);
        this.pointer += 2;
        return value;
    }

    public setU16(value: number) {
        this.dataView.setUint16(this.pointer, value);
        this.pointer += 2;
    }

    public getU32(): number {
        const value = this.dataView.getUint32(this.pointer);
        this.pointer += 4;
        return value;
    }

    public setU32(value: number) {
        this.dataView.setUint32(this.pointer, value);
        this.pointer += 4;
    }

    public getI32(): number {
        const value = this.dataView.getInt32(this.pointer);
        this.pointer += 4;
        return value;
    }

    public setI32(value: number) {
        this.dataView.setInt32(this.pointer, value);
        this.pointer += 4;
    }

    public getU64(): bigint {
        const value = this.dataView.getBigUint64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public setU64(value: bigint) {
        this.dataView.setBigUint64(this.pointer, value);
        this.pointer += 8;
    }

    public getI64(): bigint {
        const value = this.dataView.getBigInt64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public setI64(value: bigint) {
        this.dataView.setBigInt64(this.pointer, value);
        this.pointer += 8;
    }

    public getBuffer(): Uint8Array {
        const size = this.getU32();
        const result = this
            .getUint8Memory()
            .subarray(
                this.ptr + this.pointer,
                this.ptr + this.pointer + size
            );

        this.pointer += size;
        return result;
    }

    public setBuffer(buffer: Uint8Array) {
        const size = buffer.length;
        this.setU32(size);

        const subbugger = this
            .getUint8Memory()
            .subarray(
                this.ptr + this.pointer,
                this.ptr + this.pointer + size
            );

        subbugger.set(buffer);

        this.pointer += size;
    }

    public getString(): string {
        return decoder.decode(this.getBuffer());
    }

    public setString(value: string) {
        const buffer = encoder.encode(value);
        this.setBuffer(buffer);
    }
}

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
    | { type: 'object', value: ListItemMapType };

interface ListItemMapType {
    [key: string]: JsValueType
}

//https://github.com/unsplash/unsplash-js/pull/174
// export type AnyJson = boolean | number | string | null | JsonArray | JsonMap;
// export interface JsonMap { [key: string]: AnyJson }
// export interface JsonArray extends Array<AnyJson> {}


const argumentsDecodeItem = (cursor: BufferCursor): JsValueType => {
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
            out.push(argumentsDecodeItem(cursor))
        }

        return out;
    }

    if (typeParam === 12) {
        const out: Record<string, JsValueType> = {};

        const listSize = cursor.getU16();

        for (let i=0; i<listSize; i++) {
            const key = cursor.getString();
            const value = argumentsDecodeItem(cursor);
            out[key] = value;
        }

        return {
            type:'object',
            value: out
        };
    }

    console.error('typeParam', typeParam);
    throw Error('Nieprawidłowe odgałęzienie');
};

export const argumentsDecode = (getUint8Memory: () => Uint8Array, ptr: number, size: number): JsValueType => {
    try {
        const cursor = new BufferCursor(getUint8Memory, ptr, size);
        return argumentsDecodeItem(cursor);
    } catch (err) {
        console.error(err);
        return [];
    }
};

export namespace Guard {
    export const isString = (value: JsValueType): value is string => {
        return typeof value === 'string';
    }

    export const isStringOrNull = (value: JsValueType): value is string | null => {
        return value === null || typeof value === 'string';
    }

    export const isNumber = (value: JsValueType): value is { type: 'u32', value: number } | { type: 'i32', value: number } => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === 'i32' || value.type === 'u32'
        }

        return false;
    }

    export const isBigInt = (value: JsValueType): value is { type: 'u64', value: bigint } | { type: 'i64', value: bigint } => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === 'i64' || value.type === 'u64'
        }

        return false;
    }
}

const assertNever = (_value: never) => {
    throw Error("assert never");
}

const getStringSize = (value: string): number => {
    return new TextEncoder().encode(value).length;
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
        return 5;   //1 + 4
    }

    if (value.type === 'i64' || value.type === 'u64') {
        return 9;   //1 + 8
    }

    if (value.type === 'object') {
        let sum = 1 + 2;

        for (const [key, propertyValue] of Object.entries(value.value)) {
            sum += getStringSize(key);
            sum += getSize(propertyValue);
        }

        return sum;
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

    return assertNever(value);
};

export const saveToBuffer = (
    getUint8Memory: () => Uint8Array,
    alloc: (size: number) => number,
    value: JsValueType,
): number => {
    const size = getSize(value);
    const ptr = alloc(size);

    const cursor = new BufferCursor(getUint8Memory, ptr, size);
    saveToBufferItem(value, cursor);

    return ptr;
};

export class JsValueBuilder {
    private readonly params: Array<JsValueType>;

    constructor(
        private readonly getUint8Memory: () => Uint8Array,
        private readonly alloc: (size: number) => number,
    ) {
        this.params = [];
    }

    public push_string(value: string) {
        this.params.push(value);
    }

    public push_buffer(buf: Uint8Array) {
        this.params.push(buf);
    }

    public push_u32(value: number) {
        this.params.push({
            type: 'u32',
            value
        });
    }

    public push_i32(value: number) {
        this.params.push({
            type: 'i32',
            value
        });
    }

    public push_u64(value: bigint) {
        this.params.push({
            type: 'u64',
            value
        });
    }

    public push_i64(value: bigint) {
        this.params.push({
            type: 'i64',
            value
        });
    }

    public push_null() {
        this.params.push(null);
    }

    public push_bool(value: boolean) {
        this.params.push(value);
    }

    public push_list(build: (list: JsValueBuilder) => void) {
        const sub_params = new JsValueBuilder(this.getUint8Memory, this.alloc);
        build(sub_params);
        this.params.push(sub_params.params);
    }

    public saveToBuffer(): number {
        return saveToBuffer(this.getUint8Memory, this.alloc, this.params);
    }

    public saveListItem(value: JsValueType): number {
        return saveToBuffer(this.getUint8Memory, this.alloc, value);
    }

    public debug() {
        console.info('debug budowania listy', this.params);
    }
}

export const convertFromListItem = (value: JsValueType): unknown => {
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
        const result: Record<string, unknown> = {};

        for (const [key, propertyValue] of Object.entries(value.value)) {
            result[key] = convertFromListItem(propertyValue);
        }

        return result;
    }

    return assertNever(value);
};

export const convertToListItem = (value: unknown): JsValueType => {
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
