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

    public set = (
        cname: string,
        cvalue: string,
        expires_in: bigint,
    ) => {
        const cvalueEncoded = cvalue == null ? "" : encodeURIComponent(cvalue);

        const d = new Date();
        d.setTime(d.getTime() + (Number(expires_in) * 1000));
        let expires = "expires="+ d.toUTCString();

        document.cookie = `${cname}=${cvalueEncoded};${expires};path=/;samesite=strict"`;
    }
}
