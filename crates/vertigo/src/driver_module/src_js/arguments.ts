///https://javascript.info/arraybuffer-binary-arrays#dataview

import { BaseExportType } from "./wasm_init";

class BufferCursor {
    private pointer: number = 0;

    constructor(
        private dataView: DataView,
        private getUint8Memory: () => Uint8Array,
    ) {
    }

    public getByte(): number {
        const value = this.dataView.getUint8(this.pointer);
        this.pointer += 1;
        return value;
    }

    public getU16(): number {
        const value = this.dataView.getUint16(this.pointer);
        this.pointer += 2;
        return value;
    }

    public getU32(): number {
        const value = this.dataView.getUint32(this.pointer);
        this.pointer += 4;
        return value;
    }

    public getI32(): number {
        const value = this.dataView.getInt32(this.pointer);
        this.pointer += 4;
        return value;
    }

    public getU64(): BigInt {
        const value = this.dataView.getBigUint64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public getI64(): BigInt {
        const value = this.dataView.getBigInt64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public getString(): string {
        const ptr = this.getU32();
        const size = this.getU32();

        const m = this.getUint8Memory().subarray(ptr, ptr + size);
        return decoder.decode(m);
    }

    public getBuffer(): Uint8Array {
        const ptr = this.getU32();
        const size = this.getU32();

        return this.getUint8Memory().subarray(ptr, ptr + size);
    }
}

const decoder = new TextDecoder("utf-8");

export type ListItemType = string | number | BigInt | boolean | null | undefined | Array<ListItemType> | Uint8Array;

const argumentsDecodeItem = (cursor: BufferCursor): ListItemType => {
    const typeParam = cursor.getByte();

    if (typeParam === 1) {
        return '';
    }

    if (typeParam === 2 || typeParam === 3 || typeParam === 4) {
        return cursor.getString();
    }

    if (typeParam === 5) {
        return cursor.getU32();
    }    

    if (typeParam === 6) {
        return cursor.getI32();
    }

    if (typeParam === 7) {
        return cursor.getU64();
    }

    if (typeParam === 8) {
        return cursor.getI64();
    }

    if (typeParam === 9) {
        return true;
    }

    if (typeParam === 10) {
        return false;
    }

    if (typeParam === 11) {
        return null;
    }

    if (typeParam === 12) {
        return undefined;
    }

    if (typeParam === 13) {
        const out: Array<ListItemType> = [];
        
        const listSize = cursor.getU16();

        for (let i=0; i<listSize; i++) {
            out.push(argumentsDecodeItem(cursor))
        }

        return out;
    }

    if (typeParam === 14) {
        return cursor.getBuffer();
    }

    console.error('typeParam', typeParam);
    throw Error('Nieprawidłowe odgałęzienie');
};

export const argumentsDecode = (getUint8Memory: () => Uint8Array, ptr: number): ListItemType => {
    try {
        const view = new DataView(getUint8Memory().buffer, ptr);
        const cursor = new BufferCursor(view, getUint8Memory);
        return argumentsDecodeItem(cursor);
    } catch (err) {
        console.error(err);
        return [];
    }
};

export namespace Guard {
    export const isString = (value: ListItemType): value is string => {
        return typeof value === 'string';
    }

    export const isStringOrNull = (value: ListItemType): value is string | null => {
        return value === null || typeof value === 'string';
    }

    export const isNumber = (value: ListItemType): value is number => {
        return typeof value === 'number';
    }

    export const isBigInt = (value: ListItemType): value is BigInt => {
        return typeof value === 'bigint';
    }
}

export class ParamListBuilder {
    private getUint8Memory: () => Uint8Array;
    private readonly exportsModule: BaseExportType;
    private readonly listId: number;

    constructor(
        getUint8Memory: () => Uint8Array,
        exportsModule: BaseExportType,
    ) {
        this.getUint8Memory = getUint8Memory;
        this.exportsModule = exportsModule;
        this.listId = exportsModule.arguments_new_list();
    }

    public debug() {
        this.exportsModule.arguments_debug(this.listId);
    }

    public push_string(value: string) {
        if (value.length === 0) {
            this.exportsModule.arguments_push_string_empty(this.listId);
        } else {
            const encoder = new TextEncoder();
            const buf = encoder.encode(value);
            let ptr = this.exportsModule.arguments_push_string_alloc(this.listId, buf.length);

            this.getUint8Memory().subarray(ptr, ptr + buf.length).set(buf);
        }
    }

    public push_buffer(buf: Uint8Array) {
        const ptr = this.exportsModule.arguments_push_buffer_alloc(this.listId, buf.length);
        this.getUint8Memory().subarray(ptr, ptr + buf.length).set(buf);
    }

    public push_u32(value: number) {
        this.exportsModule.arguments_push_u32(this.listId, value);
    }

    public push_i32(value: number) {
        this.exportsModule.arguments_push_i32(this.listId, value);
    }

    public push_u64(value: BigInt) {
        this.exportsModule.arguments_push_u64(this.listId, value);
    }

    public push_i64(value: BigInt) {
        this.exportsModule.arguments_push_i64(this.listId, value);
    }

    public push_null() {
        this.exportsModule.arguments_push_null(this.listId);
    }

    public push_bool(value: boolean) {
        if (value) {
            this.exportsModule.arguments_push_true(this.listId);
        } else {
            this.exportsModule.arguments_push_false(this.listId);
        }
    }

    public push_list(build: (list: ParamListBuilder) => void) {
        const sub_params = new ParamListBuilder(this.getUint8Memory, this.exportsModule);
        build(sub_params);
        this.exportsModule.arguments_push_sublist(this.listId, sub_params.listId);
    }

    public freeze(): number {
        this.exportsModule.arguments_freeze(this.listId);
        return this.listId;
    }
}
