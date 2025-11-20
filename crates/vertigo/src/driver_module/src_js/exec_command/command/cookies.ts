import { JsJsonType } from "../../jsjson";

export class Cookies {
    public get = (cname: string): string => {
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
                return decodeURIComponent(cookieValue);
            }
        }

        return '';
    }

    public get_json = (cname: string): JsJsonType => {
        let cvalue_str = this.get(cname);

        if (cvalue_str.length !== 0) {
            try {
                let cookie_value = JSON.parse(cvalue_str);
                return cookie_value;
            } catch (e) {
                console.error!("Error deserializing cookie", e);
            }
        }
        return null
    }

    public set = (
        cname: string,
        cvalue: string,
        expires_in: number,
    ) => {
        const cvalueEncoded = cvalue == null ? "" : encodeURIComponent(cvalue);

        const d = new Date();
        d.setTime(d.getTime() + (expires_in * 1000));
        let expires = "expires="+ d.toUTCString();

        document.cookie = `${cname}=${cvalueEncoded};${expires};path=/;samesite=strict"`;
    }

    public set_json = (
        cname: string,
        cvalue: JsJsonType,
        expires_in: number,
    ) => {
        let cvalue_str = JSON.stringify(cvalue);

        this.set(cname, cvalue_str, expires_in);
    }
}
