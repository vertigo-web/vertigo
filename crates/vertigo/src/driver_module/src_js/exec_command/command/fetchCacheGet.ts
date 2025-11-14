import { JsJsonType } from "../../jsjson";

// interface ResponseType {
//     data: string | null,
// }

export const fetchCacheGet = (): JsJsonType => {
    const cache = document.documentElement.getAttribute('data-fetch-cache');

    return {
        data: cache
    };
};

