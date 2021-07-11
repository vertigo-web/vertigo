export class DriverBrowserIntervalJs {

    private readonly callback: (callback_id: number) => void;

    constructor(callback: (callback_id: number) => void) {
        this.callback = callback;
    }

    public set_interval(duration: number, callback_id: number): number {
        const timer_id = setInterval(() => {
            this.callback(callback_id);
        }, duration);

        return timer_id;
    }

    public clear_interval(timer_id: number) {
        clearInterval(timer_id);
    }
}
