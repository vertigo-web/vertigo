import { Guard, ListItemType } from "../arguments";
import { Fetch } from "../module/fetch";

export const initFetch = (fetch: Fetch) => (method: string, args: Array<ListItemType>): number => {
    if (method === 'send') {
        const [requestId, httpMethod, url, headers, body, ...rest] = args;

        if (
            Guard.isNumber(requestId) &&
            Guard.isString(httpMethod) &&
            Guard.isString(url) &&
            Guard.isString(headers) &&
            Guard.isStringOrNull(body) &&
            rest.length === 0
        ) {
            fetch.fetch_send_request(requestId.value, httpMethod, url, headers, body);
            return 0;
        }

        console.error('js-call -> module -> fetch -> send: incorrect parameters', args);
    }

    console.error('js-call -> module -> fetch: incorrect parameters', args);
    return 0;
};
