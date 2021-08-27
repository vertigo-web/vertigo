export class DriverBrowserFetchJs {
    private readonly callback: (request_id: number, success: boolean, status: number, response: string) => void;

    constructor(callback: (request_id: number, success: boolean, status: number, response: string) => void) {
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
            .then((response) =>
                response.text()
                    .then((responseText) => {
                        this.callback(request_id, true, response.status, responseText);
                    })
                    .catch((err) => {
                        console.error('fetch error (2)', err);
                        this.callback(request_id, false, response.status, new String(err).toString());
                    })
            )
            .catch((err) => {
                console.error('fetch error (1)', err);
                this.callback(request_id, false, 0, new String(err).toString());
            })
        ;
    }
}
