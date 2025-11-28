
export const getDisableHydration = (): boolean => {
    const metadataDiv = document.getElementById('v-metadata');
    const value = metadataDiv?.getAttribute('data-env-disable-hydration');
    return value === 'true';
};
