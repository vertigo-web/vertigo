export class DriverBrowserFetchJs {
    constructor(callback) {
        this.callback = callback;
    }
    send_request(request_id, method, url, headers, body) {
        const headers_record = JSON.parse(headers);
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
        });
    }
}
