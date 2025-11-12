import { JsJsonType } from "../jsjson";
import { JsValueConst } from "../jsvalue_types";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

interface FetchRequestType {
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
        response: JsJsonType,
    }
} | {
    Error: {
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

const processResponse = async (response: Response): Promise<FetchResponseType> => {
    const status = response.status;

    try {
        const json = await response.json();

        return {
            Ok: {
                status,
                response: json
            }
        };
    } catch (error) {
        return {
            Error: {
                message: String(error),
            }
        };
    }
};

export class Fetch {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public fetch_send_request = (
        callback_id: bigint,
        request: FetchRequestType
    ) => {
        this.fetch_send_request_inner(callback_id, request);
    }

    private fetch_send_request_inner = async (
        callback_id: bigint,
        request: FetchRequestType
    ): Promise<void> => {
        const wasm = this.getWasm();

        console.info('fetch request', request);

        try {
            const response = await fetch(request.url, {
                method: request.method,
                headers: getHeaders(request.headers),
                body: getBodyString(request.body),
            });

            const response2 = await processResponse(response);

            wasm.wasm_callback(callback_id, {
                type: JsValueConst.Json,
                value: response2
            });

        } catch (err) {
            console.error('fetch error (1)', err);
            const responseMessage = new String(err).toString();

            const responseToWasm: FetchResponseType = {
                'Error': {
                    message: responseMessage
                }
            };

            wasm.wasm_callback(callback_id, {
                type: JsValueConst.Json,
                value: responseToWasm
            });
        }
    }
}
