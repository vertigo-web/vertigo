// Reader for the flat DOM command format.
//
// The format is defined and documented in `crates/vertigo/src/dev/command_wire.rs`, which
// also holds the encoder and the round-trip tests. This file has to agree with it byte for
// byte; `dom_wire.test.ts` checks that against a fixture produced by that encoder.
//
//   blob     := dict commands
//   dict     := count:varint (len:varint utf8*)*
//   commands := (tag:u8 field*)*            -- to the end of the buffer
//
// Element and attribute names live in the dictionary and are referenced by index, so a name
// is decoded once per batch rather than once per command.

import { CommandType } from "./dom";

export const Tag = {
    CreateNode: 1,
    CreateText: 2,
    UpdateText: 3,
    SetAttr: 4,
    RemoveAttr: 5,
    RemoveNode: 6,
    RemoveText: 7,
    InsertBefore: 8,
    InsertCss: 9,
    CreateComment: 10,
    RemoveComment: 11,
    CallbackAdd: 12,
    CallbackRemove: 13,
} as const;

const decoder = new TextDecoder("utf-8");

export class CommandCursor {
    private at: number = 0;

    constructor(private readonly bytes: Uint8Array) {}

    public isEmpty(): boolean {
        return this.at >= this.bytes.length;
    }

    public where(): string {
        return `offset ${this.at} of ${this.bytes.length}`;
    }

    public byte(): number {
        const value = this.bytes[this.at];

        if (value === undefined) {
            throw new Error("dom command: buffer ended early");
        }

        this.at += 1;
        return value;
    }

    // LEB128. Built with multiplication rather than `<<`, which in JavaScript is a 32-bit
    // operation and would wrap on ids past four billion.
    public varint(): number {
        let value = 0;
        let scale = 1;

        while (true) {
            const byte = this.byte();
            value += (byte & 0x7f) * scale;

            if ((byte & 0x80) === 0) {
                return value;
            }

            scale *= 128;
        }
    }

    public string(): string {
        const length = this.varint();
        const end = this.at + length;

        if (end > this.bytes.length) {
            throw new Error("dom command: string runs past the end of the buffer");
        }

        // `subarray`, not `slice`: the bytes are a view straight into wasm memory and the
        // decoder copies out of them anyway.
        const text = decoder.decode(this.bytes.subarray(this.at, end));
        this.at = end;
        return text;
    }

    public name(names: Array<string>): string {
        const position = this.varint();
        const value = names[position];

        if (value === undefined) {
            throw new Error(`dom command: name index ${position} is not in the dictionary`);
        }

        return value;
    }

    // 0 means "no reference node" - see NO_ID in command_wire.rs.
    public optionalId(): number | null {
        const id = this.varint();
        return id === 0 ? null : id;
    }
}

export const readNames = (cursor: CommandCursor): Array<string> => {
    const count = cursor.varint();
    const names: Array<string> = [];

    for (let index = 0; index < count; index++) {
        names.push(cursor.string());
    }

    return names;
};

// Object form of the stream, for hydration - which runs on the first flush only and wants to
// look at the commands before they are applied. The hot path in `dom.ts` never builds these.
export const decodeCommands = (bytes: Uint8Array): Array<CommandType> => {
    const cursor = new CommandCursor(bytes);
    const names = readNames(cursor);
    const commands: Array<CommandType> = [];

    while (!cursor.isEmpty()) {
        const tag = cursor.byte();

        switch (tag) {
            case Tag.CreateNode:
                commands.push({ CreateNode: { id: cursor.varint(), name: cursor.name(names) } });
                break;
            case Tag.CreateText:
                commands.push({ CreateText: { id: cursor.varint(), value: cursor.string() } });
                break;
            case Tag.UpdateText:
                commands.push({ UpdateText: { id: cursor.varint(), value: cursor.string() } });
                break;
            case Tag.SetAttr:
                commands.push({
                    SetAttr: {
                        id: cursor.varint(),
                        name: cursor.name(names),
                        value: cursor.string(),
                    },
                });
                break;
            case Tag.RemoveAttr:
                commands.push({ RemoveAttr: { id: cursor.varint(), name: cursor.name(names) } });
                break;
            case Tag.RemoveNode:
                commands.push({ RemoveNode: { id: cursor.varint() } });
                break;
            case Tag.RemoveText:
                commands.push({ RemoveText: { id: cursor.varint() } });
                break;
            case Tag.InsertBefore:
                commands.push({
                    InsertBefore: {
                        parent: cursor.varint(),
                        child: cursor.varint(),
                        ref_id: cursor.optionalId(),
                    },
                });
                break;
            case Tag.InsertCss:
                commands.push({
                    InsertCss: {
                        selector: cursor.byte() === 0 ? null : cursor.string(),
                        value: cursor.string(),
                    },
                });
                break;
            case Tag.CreateComment:
                commands.push({ CreateComment: { id: cursor.varint(), value: cursor.string() } });
                break;
            case Tag.RemoveComment:
                commands.push({ RemoveComment: { id: cursor.varint() } });
                break;
            case Tag.CallbackAdd:
                commands.push({
                    CallbackAdd: {
                        id: cursor.varint(),
                        event_name: cursor.string(),
                        callback_id: cursor.varint(),
                    },
                });
                break;
            case Tag.CallbackRemove:
                commands.push({
                    CallbackRemove: {
                        id: cursor.varint(),
                        event_name: cursor.string(),
                        callback_id: cursor.varint(),
                    },
                });
                break;
            default:
                throw new Error(`dom command: unknown tag ${tag}`);
        }
    }

    return commands;
};
