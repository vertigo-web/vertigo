/// What of the browser's `Storage` the driver uses.
export type KeyValueStorage = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

/// The browser's `localStorage`, kept for the page's origin across visits. A browser may refuse
/// it to the page, f. ex. with its storage blocked, which throws already when it is reached, or
/// have no room left: then nothing is kept or read, and the console tells why.
export class LocalStorage {
    constructor(private readonly storage: () => KeyValueStorage) {}

    private use<T>(name: string, use: (storage: KeyValueStorage) => T): T | null {
        try {
            return use(this.storage());
        } catch (error) {
            console.warn('localStorage', name, error);
            return null;
        }
    }

    public get = (name: string): string | null => this.use(name, (storage) => storage.getItem(name));

    public set = (name: string, value: string) => {
        this.use(name, (storage) => storage.setItem(name, value));
    }

    public remove = (name: string) => {
        this.use(name, (storage) => storage.removeItem(name));
    }
}
