export class Metadata {
    private readonly metadata: HTMLElement;

    constructor() {
        const metadata = document.getElementById('v-metadata');

        if (metadata === null) {
            throw Error('Expected v-metadata');
        }

        this.metadata = metadata;
        metadata.remove();
    }

    private get = (attr: string): string | null => {
        return this.metadata.getAttribute(attr) ?? null;
    }

    getEnv(name: string) {
        return this.get(`data-env-${name}`);
    }

    getFetchCache() {
        return this.get('data-fetch-cache') ?? null;
    }

    getEnabledHydration = (): boolean => {
        const value = this.get('data-env-disable-hydration');
        return value !== 'true';
    }
}
