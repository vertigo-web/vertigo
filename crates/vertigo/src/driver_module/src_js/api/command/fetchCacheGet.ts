import { JsJsonType } from "../../jsjson";
import { Metadata } from "../metadata";

export const fetchCacheGet = (metadata: Metadata): JsJsonType => {
    const cache = metadata.getFetchCache();

    return {
        data: cache
    };
};
