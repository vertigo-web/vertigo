export class DriverBrowserIntervalJs {
    constructor(callback) {
        this.callback = callback;
    }
    set_interval(duration, callback_id) {
        const timer_id = setInterval(() => {
            this.callback(callback_id);
        }, duration);
        return timer_id;
    }
    clear_interval(timer_id) {
        clearInterval(timer_id);
    }
}
