type ResolveFn<T> = (data: T) => void;
type RejectFn = (err: unknown) => void;

interface PromiseResolveReject<T> {
    readonly resolve: (value: T) => void,
    readonly reject: (err: unknown) => void,
};

const createPromiseValue = <T>(): [PromiseResolveReject<T>, Promise<T>] => {
    let resolve: ResolveFn<T> | null = null;
    let reject: RejectFn | null = null;

    const promise: Promise<T> = new Promise((localResolve: ResolveFn<T>, localReject: RejectFn) => {
        resolve = localResolve;
        reject = localReject;
    });

    if (resolve === null) {
        throw Error('createPromiseValue - resolve is null');
    }

    if (reject === null) {
        throw Error('createPromiseValue - reject is null');
    }

    const promiseValue = {
        resolve,
        reject,
    };

    return [promiseValue, promise];
};

export class PromiseBoxRace<T> {
    private promiseResolveReject: PromiseResolveReject<T> | null = null;
    readonly promise: Promise<T>;

    constructor() {
        const [promiseResolveReject, promise] = createPromiseValue<T>();

        this.promiseResolveReject = promiseResolveReject;
        this.promise = promise;
    }

    resolve = (value: T) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;

        if (promiseResolveReject === null) {
            return;
        }

        promiseResolveReject.resolve(value);
    }

    reject = (err?: unknown) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;

        if (promiseResolveReject === null) {
            return;
        }

        promiseResolveReject.reject(err);
    }

    isFulfilled = (): boolean => {
        return this.promiseResolveReject === null;
    }
}
