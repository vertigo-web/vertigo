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

type FetchResponseType = {
    Ok: {
        status: number,
        response: {
            Text: string
        } | {
            Json: JsJsonType,
        }
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

const processResponse = async (response: Response): Promise<FetchResponseType> => {
    const status = response.status;
    const contentType = response.headers.get("Content-Type");

    try {
        if (contentType?.startsWith('text/plain;')) {
            return {
                Ok: {
                    status,
                    response: {
                        Text: await response.text(),
                    }
                }
            }
        }

        const json = parseJsonBody(await response.text());

        return {
            Ok: {
                status,
                response: {
                    Json: json
                }
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

    try {
        const response = await fetch(request.url, {
            method: request.method,
            headers: getHeaders(request.headers),
            body: getBodyString(request.body),
        });

        const response2 = await processResponse(response);

        wasm.wasmCommand({
            'FetchExecResponse': {
                response: response2,
                callback: callback_id,
            }
        });

    } catch (err) {
        console.error('fetch error (1)', err);
        const responseMessage = new String(err).toString();

        const responseToWasm: FetchResponseType = {
            'Err': {
                message: responseMessage
            }
        };

        wasm.wasmCommand({
            'FetchExecResponse': {
                response: responseToWasm,
                callback: callback_id,
            }
        });
    }
};


