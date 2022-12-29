import { JsJsonType } from "../jsjson";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

const getTypeResponse = (contentType: string | null): 'json' | 'text' | 'bin' => {
    if (contentType === null) {
        console.error('Missing header content-type');
        return 'bin';
    }

    const [type] = contentType.split(";");

    if (type === undefined) {
        console.error('Missing value for content-type');
        return 'bin';
    }

    const typeClear = type.toLowerCase().trim();

    if (typeClear === 'application/json') {
        return 'json';
    }

    if (typeClear === 'text/plain') {
        return 'text';
    }

    console.error(`No match found for content-type=${contentType}`);
    return 'bin';
}

const catchError = async (
    wasm: ModuleControllerType<ExportType>,
    callback_id: bigint,
    response: Response,
    callbackSuccess: (response: Response) => Promise<void>
) => {
    try {
        await callbackSuccess(response);
    } catch (error) {
        console.error('fetch error (2) - json', error);
        const responseMessage = new String(error).toString();

        wasm.wasm_callback(callback_id, [
            false,                                      //ok
            { type: 'u32', value: response.status },    //http code
            responseMessage                             //body (string)
        ]);
    }
};

const getHeadersAndBody = (headersRecord: Record<string, string>, body: undefined | string | Uint8Array | JsJsonType | undefined): [Headers, string | ArrayBuffer | undefined] => {
    const headers = new Headers(headersRecord);

    if (body === undefined) {
        return [
            headers,
            undefined
        ]
    }

    if (typeof body === 'string') {
        if (headers.has('content-type') === false) {
            headers.set('content-type', 'text/plain; charset=utf-8');
        }

        return [
            headers,
            body
        ];
    }


    if (body instanceof Uint8Array) {
        return [
            headers,
            body
        ];
    }

    //JsJsonType
    if (headers.has('content-type') === false) {
        headers.set('content-type', 'application/json; charset=utf-8');
    }

    return [
        headers,
        JSON.stringify(body),
    ];
};

export class Fetch {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public fetch_send_request = (
        callback_id: bigint,
        method: string,
        url: string,
        headers: Record<string, string>,
        body: string | Uint8Array | JsJsonType | undefined,
    ) => {
        this.fetch_send_request_inner(callback_id, method, url, headers, body);
    }

    private fetch_send_request_inner = async (
        callback_id: bigint,
        method: string,
        url: string,
        headers: Record<string, string>,
        body: string | Uint8Array | JsJsonType | undefined,
    ): Promise<void> => {
        const wasm = this.getWasm();

        const [fetchHeaders, fetchBody] = getHeadersAndBody(headers, body);

        try {
            const response = await fetch(url, {
                method,
                headers: fetchHeaders,
                body: fetchBody,
            });

            const contentType = response.headers.get('content-type');
            const responseType = getTypeResponse(contentType);
        
            if (responseType === 'json') {
                catchError(wasm, callback_id, response, async (response) => {
                    const json = await response.json();

                    wasm.wasm_callback(callback_id, [
                        true,                                       //ok
                        { type: 'u32', value: response.status },    //http code
                        {                                           //body (json)
                            type: 'json',
                            value: json
                        }
                    ]);
                });
                return;
            }

            if (responseType === 'text') {
                catchError(wasm, callback_id, response, async (response) => {
                    const text = await response.text();

                    wasm.wasm_callback(callback_id, [
                        true,                                       //ok
                        { type: 'u32', value: response.status },    //http code
                        text                                        //body (text)
                    ]);
                });
                return;
            }

            catchError(wasm, callback_id, response, async (response) => {
                const text = await response.arrayBuffer();
                const textUunt8Array = new Uint8Array(text);

                wasm.wasm_callback(callback_id, [
                    true,                                       //ok
                    { type: 'u32', value: response.status },    //http code
                    textUunt8Array                              //body (text)
                ]);
            });
        } catch (err) {
            console.error('fetch error (1)', err);
            const responseMessage = new String(err).toString();

            wasm.wasm_callback(callback_id, [
                false,                                      //ok
                { type: 'u32', value: 0 },                  //http code
                responseMessage                             //body (string)
            ]);
        }
    }
}
