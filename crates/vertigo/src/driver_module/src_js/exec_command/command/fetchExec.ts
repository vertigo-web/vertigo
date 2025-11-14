import { JsJsonType } from "../../jsjson";
import { ModuleControllerType } from "../../wasm_init";
import { ExportType } from "../../wasm_module";

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


export const fetchExec = async (
    getWasm: () => ModuleControllerType<ExportType>,
    callback_id: bigint,
    request: FetchRequestType
): Promise<void> => {
    const wasm = getWasm();

    console.info('fetch request', request);

    try {
        const response = await fetch(request.url, {
            method: request.method,
            headers: getHeaders(request.headers),
            body: getBodyString(request.body),
        });

        const response2 = await processResponse(response);

        wasm.wasm_command({
            'FetchExecResponse': {
                response: response2,
                callback: Number(callback_id),
            }
        });

    } catch (err) {
        console.error('fetch error (1)', err);
        const responseMessage = new String(err).toString();

        const responseToWasm: FetchResponseType = {
            'Error': {
                message: responseMessage
            }
        };

        wasm.wasm_command({
            'FetchExecResponse': {
                response: responseToWasm,
                callback: Number(callback_id),
            }
        });
    }
};


