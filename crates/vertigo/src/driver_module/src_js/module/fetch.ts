import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class Fetch {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public fetch_send_request = (
        request_id: number,
        method_ptr: BigInt,
        method_len: BigInt,
        url_ptr: BigInt,
        url_len: BigInt,
        headers_ptr: BigInt,
        headers_len: BigInt,
        body_ptr: BigInt,
        body_len: BigInt,
    ) => {
        const wasm = this.getWasm();

        const method = wasm.decodeText(method_ptr, method_len);
        const url = wasm.decodeText(url_ptr, url_len);
        const headers = wasm.decodeText(headers_ptr, headers_len);
        const body = wasm.decodeTextNull(body_ptr, body_len);

        const headers_record: Record<string, string> = JSON.parse(headers);

        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) =>
                response.text()
                    .then((responseText) => {
                        wasm.pushString(responseText);
                        wasm.exports.fetch_callback(request_id, 1, response.status);
                    })
                    .catch((err) => {
                        console.error('fetch error (2)', err);
                        const responseMessage = new String(err).toString();
                        wasm.pushString(responseMessage);
                        wasm.exports.fetch_callback(request_id, 0, response.status);
                    })
            )
            .catch((err) => {
                console.error('fetch error (1)', err);
                const responseMessage = new String(err).toString();
                wasm.pushString(responseMessage);
                wasm.exports.fetch_callback(request_id, 0, 0);
            })
        ;
    }
}
