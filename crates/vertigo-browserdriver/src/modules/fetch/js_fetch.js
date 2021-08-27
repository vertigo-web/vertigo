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
            .then((response) => response.text()
            .then((responseText) => {
            this.callback(request_id, true, response.status, responseText);
        })
            .catch((err) => {
            console.error('fetch error (2)', err);
            this.callback(request_id, false, response.status, new String(err).toString());
        }))
            .catch((err) => {
            console.error('fetch error (1)', err);
            this.callback(request_id, false, 0, new String(err).toString());
        });
    }
}
