import { JsJsonType } from "../../jsjson";

// interface ResponseType {
//     data: string | null,
// }

export const fetchCacheGet = (): JsJsonType => {
    const cache = document.getElementById('v-metadata')?.getAttribute('data-fetch-cache') ?? null;

    return {
        data: cache
    };
};
