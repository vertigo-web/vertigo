import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class Fetch {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public fetch_send_request = (
        callback_id: bigint,
        method: string,
        url: string,
        headers: string,
        body: string | null,
    ) => {
        const wasm = this.getWasm();

        const headers_record: Record<string, string> = JSON.parse(headers);

        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) =>
                response.text()
                    .then((responseText) => {
                        const responseJson = JSON.parse(responseText);

                        wasm.wasm_callback(callback_id, [
                            true,                                       //ok
                            { type: 'u32', value: response.status },    //http code
                            {                                           //body
                                type: 'json',
                                value: responseJson
                            }
                        ]);
                    })
                    .catch((err) => {
                        console.error('fetch error (2)', err);
                        const responseMessage = new String(err).toString();

                        wasm.wasm_callback(callback_id, [
                            false,                                      //ok
                            { type: 'u32', value: response.status },    //http code
                            {                                           //body
                                type: 'json',
                                value: {
                                    error_message: responseMessage
                                }
                            }
                        ]);
                    })
            )
            .catch((err) => {
                console.error('fetch error (1)', err);
                const responseMessage = new String(err).toString();

                wasm.wasm_callback(callback_id, [
                    false,                                      //ok
                    { type: 'u32', value: 0 },                  //http code
                    {                                           //body
                        type: 'json',
                        value: {
                            error_message: responseMessage
                        }
                    }
                ]);
            })
        ;
    }
}
