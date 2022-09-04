import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class Fetch {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public fetch_send_request = (
        request_id: number,
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
                        const new_params = this.getWasm().newList();
                        new_params.push_u32(request_id);            //request_id
                        new_params.push_bool(true);                 //ok
                        new_params.push_u32(response.status);       //http code
                        new_params.push_string(responseText);       //body
                        let params_id = new_params.saveToBuffer();

                        wasm.exports.fetch_callback(params_id);
                    })
                    .catch((err) => {
                        console.error('fetch error (2)', err);
                        const responseMessage = new String(err).toString();

                        const new_params = this.getWasm().newList();
                        new_params.push_u32(request_id);            //request_id
                        new_params.push_bool(false);                //ok
                        new_params.push_u32(response.status);       //http code
                        new_params.push_string(responseMessage);    //body
                        let params_id = new_params.saveToBuffer();

                        wasm.exports.fetch_callback(params_id);
                    })
            )
            .catch((err) => {
                console.error('fetch error (1)', err);
                const responseMessage = new String(err).toString();

                const new_params = this.getWasm().newList();
                new_params.push_u32(request_id);                    //request_id
                new_params.push_bool(false);                        //ok
                new_params.push_u32(0);                             //http code
                new_params.push_string(responseMessage);            //body
                let params_id = new_params.saveToBuffer();

                wasm.exports.fetch_callback(params_id);
            })
        ;
    }
}
