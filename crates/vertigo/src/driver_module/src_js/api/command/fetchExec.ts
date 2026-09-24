import { JsJsonType } from "../../jsjson";
import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";
import { CallbackId } from "../types";

export interface FetchRequestType {
    method: string,
    url: string,
    headers: Array<{ k: string, v: string }>,
    body: 'None' | {
        Data: {
            data: JsJsonType
        }
    }
}

type FetchResponseContent = {
    Text: string
} | {
    Json: JsJsonType,
};

type FetchResponseType = {
    Ok: {
        status: number,
        response: FetchResponseContent,
    }
} | {
    Err: {
        message: string,
    }
};

const getHeaders = (headers: Array<{ k: string, v: string }>): Record<string, string> => {
    const result: Record<string, string> = {};

    for (const { k, v } of headers) {
        result[k] = v;
    }

    return result;
};

const getBodyString = (body: FetchRequestType['body']): string | undefined => {
    if (body === 'None') {
        return undefined;
    }

    return JSON.stringify(body.Data.data);
};

// 204/205 carry no body, and any other response with an empty body
// would crash response.json(). Treat both as Json: null so the caller
// sees a successful response with the real status code.
export const parseJsonBody = (bodyText: string): JsJsonType | null =>
    bodyText.length === 0 ? null : JSON.parse(bodyText);

const isTextPlain = (contentType: string | null): boolean =>
    contentType?.split(';')[0]?.trim().toLowerCase() === 'text/plain';

// A body that isn't JSON falls back to Text rather than failing, so the caller
// still gets the status code - e.g. a readiness endpoint answering a bare `OK`
// without any Content-Type. Mirrors `decode_body` in vertigo-cli's SSR fetch.
export const decodeBody = (contentType: string | null, bodyText: string): FetchResponseContent => {
    if (isTextPlain(contentType)) {
        return { Text: bodyText };
    }

    try {
        return { Json: parseJsonBody(bodyText) };
    } catch {
        return { Text: bodyText };
    }
};

const processResponse = async (response: Response): Promise<FetchResponseType> => {
    const status = response.status;
    const contentType = response.headers.get("Content-Type");

    try {
        return {
            Ok: {
                status,
                response: decodeBody(contentType, await response.text()),
            }
        };
    } catch (error) {
        return {
            Err: {
                message: String(error),
            }
        };
    }
};


export const fetchExec = async (
    getWasm: () => ModuleControllerType<ExportType>,
    callback_id: CallbackId,
    request: FetchRequestType
): Promise<void> => {
    const wasm = getWasm();

    const send = (response: FetchResponseType) => {
        wasm.wasmCommand({
            'FetchExecResponse': {
                response,
                callback: callback_id,
            }
        });
    };

    try {
        const response = await fetch(request.url, {
            method: request.method,
            headers: getHeaders(request.headers),
            body: getBodyString(request.body),
        });

        send(await processResponse(response));
    } catch (err) {
        console.error('fetch error (1)', err);
        const responseMessage = new String(err).toString();

        send({
            'Err': {
                message: responseMessage
            }
        });
    }
};


