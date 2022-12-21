import { JsValueType } from "./jsvalue";

export namespace GuardJsValue {
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
