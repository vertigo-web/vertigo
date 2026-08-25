// The other half of the cross-language pin for the DOM wire format.
//
// The byte array below is produced by the Rust encoder and asserted there too, by
// `matches_the_cross_language_fixture` in `crates/vertigo/src/dev/command_wire.rs`. Each side
// round-trips against itself, which only proves each is self-consistent; this is what stops
// the two agreeing on different formats. Change the format and one of these two tests fails.

import { decodeCommands } from "./dom_wire";
import { CommandType } from "./dom";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

const FIXTURE = new Uint8Array([
    2, 3, 100, 105, 118, 5, 99, 108, 97, 115, 115, 1, 4, 0, 4, 4, 1, 3, 114, 111, 119, 1,
    172, 2, 0, 5, 172, 2, 1, 2, 5, 15, 122, 97, 197, 188, 195, 179, 197, 130, 196, 135, 32,
    240, 159, 166, 128, 3, 5, 0, 8, 1, 4, 240, 162, 4, 8, 1, 4, 0, 9, 1, 2, 46, 97, 9, 99,
    111, 108, 111, 114, 58, 114, 101, 100, 9, 0, 14, 64, 109, 101, 100, 105, 97, 32, 112,
    114, 105, 110, 116, 123, 125, 10, 6, 3, 114, 111, 119, 11, 6, 6, 4, 7, 5, 12, 4, 5, 99,
    108, 105, 99, 107, 77, 13, 4, 5, 99, 108, 105, 99, 107, 172, 2,
]);

const EXPECTED: Array<CommandType> = [
    { CreateNode: { id: 4, name: 'div' } },
    { SetAttr: { id: 4, name: 'class', value: 'row' } },
    { CreateNode: { id: 300, name: 'div' } },
    { RemoveAttr: { id: 300, name: 'class' } },
    { CreateText: { id: 5, value: 'zażółć 🦀' } },
    { UpdateText: { id: 5, value: '' } },
    { InsertBefore: { parent: 1, child: 4, ref_id: 70000 } },
    { InsertBefore: { parent: 1, child: 4, ref_id: null } },
    { InsertCss: { selector: '.a', value: 'color:red' } },
    { InsertCss: { selector: null, value: '@media print{}' } },
    { CreateComment: { id: 6, value: 'row' } },
    { RemoveComment: { id: 6 } },
    { RemoveNode: { id: 4 } },
    { RemoveText: { id: 5 } },
    { CallbackAdd: { id: 4, event_name: 'click', callback_id: 77 } },
    { CallbackRemove: { id: 4, event_name: 'click', callback_id: 300 } },
];

console.log("\n--- Test dom wire: cross-language fixture ---");

const decoded = decodeCommands(FIXTURE);

assert(decoded.length === EXPECTED.length,
    `decoded ${decoded.length} commands, expected ${EXPECTED.length}`);

for (let index = 0; index < EXPECTED.length; index++) {
    const got = JSON.stringify(decoded[index]);
    const want = JSON.stringify(EXPECTED[index]);
    assert(got === want, `command ${index}: ${want}${got === want ? '' : ` (got ${got})`}`);
}

console.log("\n--- Test dom wire: interesting values ---");

// A multi-byte varint. 70000 needs three LEB128 bytes, and JavaScript's `<<` would have
// wrapped it; 300 needs two, which catches an off-by-one in the continuation bit.
const insert = decoded[6] as { InsertBefore: { ref_id: number | null } };
assert(insert.InsertBefore.ref_id === 70000, "three-byte varint decodes exactly");

const removeAttr = decoded[3] as { RemoveAttr: { id: number } };
assert(removeAttr.RemoveAttr.id === 300, "two-byte varint decodes exactly");

// Non-ASCII: the length prefix counts bytes, the decoder must produce characters.
const text = decoded[4] as { CreateText: { value: string } };
assert(text.CreateText.value === 'zażółć 🦀', "utf-8 survives the length prefix");

// Both names appear twice in the fixture and are stored once in the dictionary.
const first = decoded[0] as { CreateNode: { name: string } };
const second = decoded[2] as { CreateNode: { name: string } };
assert(first.CreateNode.name === 'div' && second.CreateNode.name === 'div',
    "a repeated name resolves through the dictionary both times");

// Absent optionals are null, not undefined or zero.
const noRef = decoded[7] as { InsertBefore: { ref_id: number | null } };
assert(noRef.InsertBefore.ref_id === null, "InsertBefore with no reference node is null");

const noSelector = decoded[9] as { InsertCss: { selector: string | null } };
assert(noSelector.InsertCss.selector === null, "InsertCss with no selector is null");

console.log("\n--- Test dom wire: malformed input ---");

const throws = (bytes: Array<number>, message: string) => {
    try {
        decodeCommands(new Uint8Array(bytes));
        assert(false, message);
    } catch {
        assert(true, message);
    }
};

throws([0, 200], "unknown tag is rejected");
throws([0, 2, 4, 9], "string longer than the buffer is rejected");
throws([0, 1, 4, 0], "name index with an empty dictionary is rejected");
throws([0, 1], "buffer ending mid-command is rejected");
