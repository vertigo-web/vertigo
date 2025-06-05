import { JsValueType, JsValueConst } from "./jsvalue_types";

export namespace GuardJsValue {
    export const isString = (value: JsValueType): value is string => {
        return typeof value === 'string';
    }

    export const isStringOrNull = (value: JsValueType): value is string | null => {
        return value === null || typeof value === 'string';
    }

    export const isNumber = (value: JsValueType): value is { type: typeof JsValueConst.U32, value: number } | { type: typeof JsValueConst.I32, value: number } => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === JsValueConst.I32 || value.type === JsValueConst.U32
        }

        return false;
    }

    export const isBigInt = (value: JsValueType): value is { type: typeof JsValueConst.U64, value: bigint } | { type: typeof JsValueConst.I64, value: bigint } => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === JsValueConst.I64 || value.type === JsValueConst.U64
        }

        return false;
    }
}
