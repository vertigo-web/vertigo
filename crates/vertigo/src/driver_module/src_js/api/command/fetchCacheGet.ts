import { JsJsonType } from "../../jsjson";
import { getMetaData } from "../metadata";

// interface ResponseType {
//     data: string | null,
// }

export const fetchCacheGet = (): JsJsonType => {
    const cache = getMetaData('data-fetch-cache') ?? null;

    return {
        data: cache
    };
};
