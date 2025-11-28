export class EventEmitter<T> {
    private events: Set<(param: T) => void>;

    constructor() {
        this.events = new Set()
    }

    on(callback: (param: T) => void) {
        let isActive = true;

        const onExec = (param: T) => {
            if (isActive) {
                callback(param);
            }
        };

        this.events.add(onExec);

        return () => {
            isActive = false;
            this.events.delete(onExec);
        };
    }

    trigger(param: T) {
        const eventsCopy = Array.from(this.events.values())

        for (const itemCallbackToRun of eventsCopy) {
            try {
                itemCallbackToRun(param);
            } catch (err) {
                console.error(err);
            }
        }
    }

    get size(): number {
        return this.events.size;
    }
}
