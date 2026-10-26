export class PromiseBoxRace<T> {
    private resolveFirst: ((value: T) => void) | null = null;
    readonly promise: Promise<T>;

    constructor() {
        this.promise = new Promise((resolve) => {
            this.resolveFirst = resolve;
        });
    }

    resolve = (value: T) => {
        const resolve = this.resolveFirst;
        this.resolveFirst = null;
        resolve?.(value);
    }

    isFulfilled = (): boolean => {
        return this.resolveFirst === null;
    }
}
