import { CallbackId } from "../types";

export interface LocationCommonType {
    add: (callback_id: CallbackId) => void,
    remove: (callback_id: CallbackId) => void,
    push: (new_hash: string) => void,
    replace: (new_hash: string) => void,
    get: () => string,
}

