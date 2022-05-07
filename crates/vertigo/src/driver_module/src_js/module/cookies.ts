import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export class Cookies {
    private readonly getWasm: () => ModuleControllerType<ExportType>;

    constructor(getWasm: () => ModuleControllerType<ExportType>) {
        this.getWasm = getWasm;
    }

    public get = (cname_ptr: BigInt, cname_len: BigInt) => {             // returns string using stack
        const wasm = this.getWasm();
        const cname = wasm.decodeText(cname_ptr, cname_len);

        for (const cookie of document.cookie.split(';')) {
            if (cookie === "") continue;

            const cookieChunk = cookie.trim().split('=');

            if (cookieChunk.length !== 2) {
                console.warn(`Cookies.get: Incorrect number of cookieChunk => ${cookieChunk.length} in ${cookie}`);
                continue;
            }

            const cookieName = cookieChunk[0];
            const cookieValue = cookieChunk[1];

            if (cookieName === undefined || cookieValue === undefined) {
                console.warn(`Cookies.get: Broken cookie part => ${cookie}`);
                continue;
            }

            if (cookieName === cname) {
                wasm.pushString(decodeURIComponent(cookieValue));
                return
            }
        }

        wasm.pushString("")
    }

    public set = (
        cname_ptr: BigInt, cname_len: BigInt,
        cvalue_ptr: BigInt, cvalue_len: BigInt,
        expires_in: BigInt,
    ) => {
        const wasm = this.getWasm();
        const cname = wasm.decodeText(cname_ptr, cname_len);
        const cvalue = wasm.decodeText(cvalue_ptr, cvalue_len);
        const cvalueEncoded = cvalue == null ? "" : encodeURIComponent(cvalue);

        const d = new Date();
        d.setTime(d.getTime() + (Number(expires_in) * 1000));
        let expires = "expires="+ d.toUTCString();

        document.cookie = `${cname}=${cvalueEncoded};${expires};path=/;samesite=strict"`;
    }
}
