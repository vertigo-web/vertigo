// The languages the user prefers, as the driver tells them to wasm.

import { languages } from "./languages";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const same = (actual: Array<string>, expected: Array<string>) =>
    JSON.stringify(actual) === JSON.stringify(expected);

assert(
    same(languages({ languages: ['pl-PL', 'pl', 'en-US', 'en'], language: 'pl-PL' }), ['pl-PL', 'pl', 'en-US', 'en']),
    'the list, the most preferred first',
);

const list = ['de'];
const copied = languages({ languages: list });
copied.push('en');
assert(same(list, ['de']), 'a copy of the list, which the browser keeps frozen');

assert(same(languages({ languages: [], language: 'fr' }), ['fr']), 'just the language without a list');
assert(same(languages({ language: 'fr' }), ['fr']), 'just the language in a browser that has no list');
assert(same(languages({}), []), 'nothing when the browser tells nothing');
