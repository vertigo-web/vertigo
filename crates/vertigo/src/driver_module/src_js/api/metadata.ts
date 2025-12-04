interface MetaDataCarrier {
    vMetadata: Element | undefined;
}

export const getMetaData = (attr: string): string | null => {
    const metadata = document.getElementById('v-metadata') ?? (window as unknown as MetaDataCarrier).vMetadata;
    return metadata?.getAttribute(attr) ?? null;
}

export const getDisableHydration = (): boolean => {
    const value = getMetaData('data-env-disable-hydration');
    return value === 'true';
};

export const trySaveMetaData = (node: ChildNode) => {
    if (node instanceof Element && node.id === 'v-metadata') {
        node.removeAttribute('data-fetch-cache');
        (window as unknown as MetaDataCarrier).vMetadata = node;
    }
};
