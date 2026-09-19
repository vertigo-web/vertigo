// Round-trip cover for the JsJson codec.
//
// This is the single channel every JS->wasm message goes through - every callback, every fetch
// response, every websocket frame, every `dom_access` reply - and until this file it had no
// test at all. A bug here is silent corruption on the Rust side rather than a crash, so the
// interesting assertion is not just "it decodes back" but "the writer used exactly the number
// of bytes the sizing pass promised": the two are separate walks over the value, and the
// allocation is made from the first.

import { BufferCursor } from "./buffer_cursor";
import { jsJsonDecodeItem, jsJsonGetSize, saveJsJsonToBufferItem, JsJsonType } from "./jsjson";

function assert(condition: boolean, message: string) {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
}

/// `BufferCursor` reads `(ptr << 32) | size` out of one bigint - the same packing wasm uses.
const longPtr = (ptr: number, size: number): bigint =>
    (BigInt(ptr) << 32n) + BigInt(size);

/// A stand-in for wasm linear memory. The block starts at `PTR` rather than 0 so that a
/// cursor confusing the block offset with the buffer offset shows up.
const PTR = 8;

function equal(left: unknown, right: unknown): boolean {
    if (left instanceof Uint8Array || right instanceof Uint8Array) {
        if (!(left instanceof Uint8Array) || !(right instanceof Uint8Array)) {
            return false;
        }
        return left.length === right.length && left.every((byte, i) => byte === right[i]);
    }

    if (Array.isArray(left) || Array.isArray(right)) {
        if (!Array.isArray(left) || !Array.isArray(right) || left.length !== right.length) {
            return false;
        }
        return left.every((item, i) => equal(item, right[i]));
    }

    if (typeof left === 'object' && left !== null && typeof right === 'object' && right !== null) {
        const leftKeys = Object.keys(left);
        const rightKeys = Object.keys(right);

        if (leftKeys.length !== rightKeys.length) {
            return false;
        }

        return leftKeys.every((key) =>
            Object.prototype.hasOwnProperty.call(right, key)
            && equal((left as Record<string, unknown>)[key], (right as Record<string, unknown>)[key])
        );
    }

    // Distinguishes undefined from null, which the codec carries as separate type ids.
    return left === right && (left === undefined) === (right === undefined);
}

function roundTrip(value: JsJsonType, message: string) {
    const size = jsJsonGetSize(value);

    // Exactly `size` bytes past PTR, so a writer that runs over the sizing pass's promise
    // throws a RangeError out of DataView rather than quietly scribbling on a neighbour.
    const memory = new Uint8Array(PTR + size);
    const getMemory = () => memory;

    saveJsJsonToBufferItem(value, new BufferCursor(getMemory, longPtr(PTR, size)));

    const readCursor = new BufferCursor(getMemory, longPtr(PTR, size));
    const decoded = jsJsonDecodeItem(readCursor);

    assert(equal(decoded, value), message);

    // And the writer must not have used *fewer* bytes than it asked for either: the reader
    // should now be exactly at the end of the block.
    let overran = false;
    try {
        readCursor.getByte();
    } catch {
        overran = true;
    }
    assert(overran, `${message} - consumed exactly ${size} bytes`);
}

console.log("\n--- Test jsjson: every variant round-trips ---");

roundTrip(true, "true");
roundTrip(false, "false");
roundTrip(null, "null");
roundTrip(undefined, "undefined");
roundTrip("", "empty string");
roundTrip("hello", "ascii string");
roundTrip("zażółć gęślą jaźń 🦀", "utf-8 string");
roundTrip(0, "zero");
roundTrip(-1.5, "negative fraction");
roundTrip(Number.MAX_SAFE_INTEGER, "max safe integer");
roundTrip(new Uint8Array([]), "empty Uint8Array");
roundTrip(new Uint8Array([0, 1, 127, 128, 255]), "Uint8Array with the byte extremes");
roundTrip([], "empty list");
roundTrip([1, "two", false, null, undefined], "mixed list");
roundTrip({}, "empty object");
roundTrip({ a: 1, b: "two" }, "flat object");

console.log("\n--- Test jsjson: nesting and awkward keys ---");

roundTrip({ "zażółć 🦀": "utf-8 key" }, "utf-8 object key");
roundTrip({ "": "empty key" }, "empty object key");
roundTrip([[[1]]], "list nested three deep");
roundTrip({ a: { b: { c: [1, { d: null }] } } }, "object nested three deep");
roundTrip(
    { name: "file.bin", data: new Uint8Array([1, 2, 3]), tags: ["a", "b"] },
    "the shape a file callback sends"
);
roundTrip(
    [[["file.txt", [104, 105]]]],
    "the shape drop/changeFile actually send - bytes as a number list"
);

console.log("\n--- Test jsjson: undefined and null stay distinct ---");

{
    const size = jsJsonGetSize(undefined);
    const memory = new Uint8Array(PTR + size);
    saveJsJsonToBufferItem(undefined, new BufferCursor(() => memory, longPtr(PTR, size)));
    const decoded = jsJsonDecodeItem(new BufferCursor(() => memory, longPtr(PTR, size)));
    assert(decoded === undefined && decoded !== null, "undefined does not decode as null");
}

{
    const size = jsJsonGetSize(null);
    const memory = new Uint8Array(PTR + size);
    saveJsJsonToBufferItem(null, new BufferCursor(() => memory, longPtr(PTR, size)));
    const decoded = jsJsonDecodeItem(new BufferCursor(() => memory, longPtr(PTR, size)));
    assert(decoded === null && decoded !== undefined, "null does not decode as undefined");
}

console.log("\n--- Test jsjson: sizing agrees with the type ids ---");

assert(jsJsonGetSize(true) === 1, "a boolean is one byte");
assert(jsJsonGetSize(null) === 1, "null is one byte");
assert(jsJsonGetSize(1.5) === 9, "a number is a tag plus f64");
assert(jsJsonGetSize("abc") === 8, "an ascii string is a tag, a u32 length and its bytes");
assert(jsJsonGetSize("ż") === 7, "a string is sized in utf-8 bytes, not code units");
assert(jsJsonGetSize(new Uint8Array([1, 2])) === 7, "a Uint8Array is a tag, a u32 length and its bytes");

console.log("\n--- Test jsjson: payloads big enough to matter ---");

{
    // A file upload is the large case in practice, and it arrives as one byte per JsJson
    // number - so a few kilobytes of file is tens of kilobytes of payload.
    const blob = new Uint8Array(5000).map((_, i) => i % 256);
    roundTrip(blob, "a 5000-byte Uint8Array");

    const asNumbers: Array<JsJsonType> = [];
    for (let i = 0; i < 2000; i++) {
        asNumbers.push(i % 256);
    }
    roundTrip([[["upload.bin", asNumbers]]], "a 2000-byte upload in the shape changeFile sends");
}
