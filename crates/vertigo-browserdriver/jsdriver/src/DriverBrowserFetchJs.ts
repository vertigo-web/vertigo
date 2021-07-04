
export class DriverBrowserFetchJs {

    private readonly callback: (request_id: number, success: boolean, response: string) => void;

    constructor(callback: (request_id: number, success: boolean, response: string) => void) {
        this.callback = callback;
    }

    public send_request(
        request_id: number,
        method: string,
        url: string,
        headers: string,
        body: string | null | undefined
    ) {

        const headers_record: Record<string, string> = JSON.parse(headers);

        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) => response.text())
            .then((response) => {
                this.callback(request_id, true, response);
            })
            .catch((err) => {
                console.error('fetch error', err);
                this.callback(request_id, false, new String(err).toString());
            })
        ;
    }
}