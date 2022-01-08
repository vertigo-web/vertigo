export 
class MapNodes<K, V> {
    private data: Map<K, V>;

    constructor() {
        this.data = new Map();
    }

    public set(key: K, value: V) {
        this.data.set(key, value);
    }

    public getItem(key: K): V | undefined {
        return this.data.get(key);
    }

    public mustGetItem(key: K): V {
        const item = this.data.get(key);

        if (item === undefined) {
            throw Error(`item not found=${key}`);
        }

        return item;
    }

    public get(label: string, key: K, callback: (value: V) => void) {
        const item = this.data.get(key);

        if (item === undefined) {
            console.error(`${label}->get: Item id not found = ${key}`);
        } else {
            callback(item);
        }
    }

    public get2(label: string, key1: K, key2: K, callback: (node1: V, node2: V) => void) {
        const node1 = this.data.get(key1);
        const node2 = this.data.get(key2);

        if (node1 === undefined) {
            console.error(`${label}->get: Item id not found = ${key1}`);
            return;
        }

        if (node2 === undefined) {
            console.error(`${label}->get: Item id not found = ${key2}`);
            return;
        }

        callback(node1, node2);
    }

    public delete(label: string, key: K, callback: (value: V) => void) {
        const item = this.data.get(key);
        this.data.delete(key);

        if (item === undefined) {
            console.error(`${label}->delete: Item id not found = ${key}`);
        } else {
            this.data.delete(key);
            callback(item);
        }
    }
}
