/// What of `navigator` tells the languages the user prefers.
export type NavigatorLanguages = {
    readonly languages?: ReadonlyArray<string>,
    readonly language?: string,
};

/// The languages the user prefers, the most preferred first: `navigator.languages`, or just
/// `navigator.language` in a browser that has no list.
export const languages = (navigator: NavigatorLanguages): Array<string> => {
    if (navigator.languages !== undefined && navigator.languages.length > 0) {
        return [...navigator.languages];
    }

    return navigator.language ? [navigator.language] : [];
};
