import { JsJsonType } from "./jsjson";

export const JsValueConst = {
    U32: 1,
    I32: 2,
    U64: 3,
    I64: 4,
    F64: 5,

    True: 6,
    False: 7,
    Null: 8,
    Undefined: 9,

    Vec: 10,
    String: 11,
    List: 12,
    Object: 13,
    Json: 14,
} as const;

export type JsValueType
    = { type: typeof JsValueConst.U32, value: number, }
    | { type: typeof JsValueConst.I32, value: number, }
    | { type: typeof JsValueConst.U64, value: bigint, }
    | { type: typeof JsValueConst.I64, value: bigint, }
    | { type: typeof JsValueConst.F64, value: number, }
    | boolean
    | null
    | undefined
    | string
    | Array<JsValueType>
    | Uint8Array
    | { type: typeof JsValueConst.Object, value: JsValueMapType }
    | { type: typeof JsValueConst.Json, value: JsJsonType };

export interface JsValueMapType {
    [key: string]: JsValueType
}
