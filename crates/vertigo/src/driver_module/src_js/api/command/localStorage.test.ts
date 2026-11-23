// The browser's `localStorage` as the driver reads and keeps it, and when the browser refuses it.

import { KeyValueStorage, LocalStorage } from "./localStorage";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

/// A storage of a map, as the browser's keeps strings
const inMemory = (): KeyValueStorage => {
    const items = new Map<string, string>();
    return {
        getItem: (name) => items.get(name) ?? null,
        setItem: (name, value) => { items.set(name, value); },
        removeItem: (name) => { items.delete(name); },
    };
};

/// The console's warnings while `run` runs, kept instead of shown
const warnings = (run: () => void): Array<unknown> => {
    const told: Array<unknown> = [];
    const warn = console.warn;
    console.warn = (...args: Array<unknown>) => { told.push(args); };
    try {
        run();
    } finally {
        console.warn = warn;
    }
    return told;
};

// The same storage every time, as `window.localStorage` is
const kept = inMemory();
const local = new LocalStorage(() => kept);

assert(local.get('theme') === null, 'nothing under a name never kept');
local.set('theme', 'dark');
assert(local.get('theme') === 'dark', 'a value kept is read back');
local.set('theme', '');
assert(local.get('theme') === '', 'an empty value is kept, not taken for nothing');
local.set('theme', 'light');
assert(local.get('theme') === 'light', 'a value kept again replaces the one before');
local.remove('theme');
assert(local.get('theme') === null, 'a value forgotten is gone');
local.remove('theme');
assert(local.get('theme') === null, 'forgetting what is gone does nothing');

// A browser with storage blocked throws already at `window.localStorage`
const blocked = new LocalStorage(() => { throw new Error('SecurityError'); });
const told = warnings(() => {
    blocked.set('theme', 'dark');
    assert(blocked.get('theme') === null, 'a blocked storage reads nothing');
    blocked.remove('theme');
});
assert(told.length === 3, 'a blocked storage is told in the console, each time');

// A full one throws on `setItem` alone
const full = new LocalStorage(() => ({
    ...kept,
    setItem: () => { throw new Error('QuotaExceededError'); },
}));
kept.setItem('old', 'kept');
assert(warnings(() => full.set('old', 'new')).length === 1, 'a full storage is told in the console');
assert(full.get('old') === 'kept', 'a full storage keeps what it had');
