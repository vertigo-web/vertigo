export class DriverBrowserHashRouteJs {
    constructor(callback) {
        window.addEventListener("hashchange", () => {
            callback(this.get_hash_location());
        }, false);
    }
    get_hash_location() {
        return location.hash.substr(1);
    }
    push_hash_location(new_hash) {
        location.hash = new_hash;
    }
}