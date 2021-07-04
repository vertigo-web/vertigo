export class DriverBrowserHashRouteJs {
    constructor(callback: (callback: string) => void) {
        window.addEventListener("hashchange", () => {
            callback(this.get_hash_location());
        }, false);
    }

    public get_hash_location(): string {
        return location.hash.substr(1);
    }

    public push_hash_location(new_hash: string) {
        location.hash = new_hash;
    }
}