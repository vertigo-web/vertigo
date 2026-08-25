//! Flat wire format for the DOM command stream.
//!
//! Every other message between wasm and the browser travels as [`JsJson`](crate::JsJson),
//! which is the right shape for a general JSON value: a tree of `BTreeMap<String, JsJson>`
//! that any `AutoJsJson` type can be built from and taken apart into. It is the wrong shape
//! for this one channel. DOM commands are emitted in the tens of thousands per update, their
//! field names are known at compile time on both sides, and encoding one as an object means
//! two `BTreeMap`s and three heap `String`s built, size-walked, serialized and dropped -
//! which measured as roughly 45% of the instructions in a "create 1000 rows" update, with
//! every field name repeated on the wire once per command.
//!
//! So the command list is encoded here instead, positionally, and travels inside the usual
//! envelope as an opaque [`JsJson::Vec`](crate::JsJson::Vec) byte blob. `JsJson` and
//! `AutoJsJson` are untouched, and keep working exactly as before for fetch bodies, `js!`
//! values, websocket messages and user types talking to external APIs.
//!
//! ## Format
//!
//! ```text
//! blob     := dict commands
//! dict     := count:varint (len:varint utf8*)*
//! commands := (tag:u8 field*)*            -- to the end of the buffer
//! ```
//!
//! Element and attribute names go in the dictionary and are referenced by index; everything
//! else is written inline. The dictionary travels with the batch rather than being a table
//! hard-coded on both sides, which is what lets a custom element or a `data-` attribute work
//! with no escape hatch, and means there is no pair of lists that can drift apart. A
//! thousand-row table has around ten distinct names, so they cost their bytes once and are a
//! one-byte index for the other thirty thousand uses. It also means the browser decodes each
//! distinct name once per batch instead of once per occurrence.
//!
//! The encoder and the decoder live in the same file on purpose: the tag numbers and the
//! field order are written down once, and the `round_trips_every_variant` test below walks
//! every variant through both.
//!
//! The TypeScript decoder in `driver_module/src_js/api/command/dom/dom.ts` is the third
//! implementation of this format, and `dom_wire.test.ts` checks it against a fixture
//! produced by the encoder here.

use std::collections::HashMap;

use crate::{
    dev::{CallbackId, command::DriverDomCommand},
    dom::dom_id::DomId,
    driver_module::StaticString,
};

/// Tag byte for each command. Append only - a number that has been used must never be reused
/// for a different shape.
mod tag {
    pub const CREATE_NODE: u8 = 1;
    pub const CREATE_TEXT: u8 = 2;
    pub const UPDATE_TEXT: u8 = 3;
    pub const SET_ATTR: u8 = 4;
    pub const REMOVE_ATTR: u8 = 5;
    pub const REMOVE_NODE: u8 = 6;
    pub const REMOVE_TEXT: u8 = 7;
    pub const INSERT_BEFORE: u8 = 8;
    pub const INSERT_CSS: u8 = 9;
    pub const CREATE_COMMENT: u8 = 10;
    pub const REMOVE_COMMENT: u8 = 11;
    pub const CALLBACK_ADD: u8 = 12;
    pub const CALLBACK_REMOVE: u8 = 13;
}

/// `InsertBefore` with no reference node. Safe as a sentinel because ids 1, 2 and 3 are
/// reserved for `html`, `head` and `body` and everything else counts up from
/// [`DomId`]'s `START_ID`, so nothing is ever zero - see the `debug_assert` in `write_id`.
const NO_ID: u64 = 0;

// --- encoding ---------------------------------------------------------------------------

/// Encode a whole batch. The result is the payload of
/// [`CommandForBrowser::DomBulkUpdate`](crate::dev::command::CommandForBrowser::DomBulkUpdate).
pub fn encode_dom_commands(commands: &[DriverDomCommand]) -> Vec<u8> {
    let mut out = Vec::with_capacity(commands.len() * 12);

    // Names first, so the decoder has the dictionary before it needs to resolve an index.
    // Collecting them up front costs one hash lookup per name and saves buffering the whole
    // command stream somewhere else while the dictionary is still being discovered.
    let mut index: HashMap<&str, u32> = HashMap::new();
    let mut names: Vec<&str> = Vec::new();

    for command in commands {
        for name in static_names(command) {
            let next = names.len() as u32;
            index.entry(name.as_str()).or_insert_with(|| {
                names.push(name.as_str());
                next
            });
        }
    }

    write_varint(&mut out, names.len() as u64);
    for name in &names {
        write_str(&mut out, name);
    }

    for command in commands {
        write_command(&mut out, command, &index);
    }

    out
}

/// The dictionary holds element and attribute names - the fields typed [`StaticString`],
/// which come from the `dom!` macro and so repeat across every row of a list.
fn static_names(command: &DriverDomCommand) -> impl Iterator<Item = &StaticString> {
    match command {
        DriverDomCommand::CreateNode { name, .. }
        | DriverDomCommand::SetAttr { name, .. }
        | DriverDomCommand::RemoveAttr { name, .. } => Some(name),
        _ => None,
    }
    .into_iter()
}

fn write_command(out: &mut Vec<u8>, command: &DriverDomCommand, index: &HashMap<&str, u32>) {
    let name_ref = |out: &mut Vec<u8>, name: &StaticString| {
        let position = index
            .get(name.as_str())
            .copied()
            // Unreachable: `static_names` visited every name in the same order.
            .unwrap_or_default();
        write_varint(out, u64::from(position));
    };

    match command {
        DriverDomCommand::CreateNode { id, name } => {
            out.push(tag::CREATE_NODE);
            write_id(out, *id);
            name_ref(out, name);
        }
        DriverDomCommand::CreateText { id, value } => {
            out.push(tag::CREATE_TEXT);
            write_id(out, *id);
            write_str(out, value);
        }
        DriverDomCommand::UpdateText { id, value } => {
            out.push(tag::UPDATE_TEXT);
            write_id(out, *id);
            write_str(out, value);
        }
        DriverDomCommand::SetAttr { id, name, value } => {
            out.push(tag::SET_ATTR);
            write_id(out, *id);
            name_ref(out, name);
            write_str(out, value);
        }
        DriverDomCommand::RemoveAttr { id, name } => {
            out.push(tag::REMOVE_ATTR);
            write_id(out, *id);
            name_ref(out, name);
        }
        DriverDomCommand::RemoveNode { id } => {
            out.push(tag::REMOVE_NODE);
            write_id(out, *id);
        }
        DriverDomCommand::RemoveText { id } => {
            out.push(tag::REMOVE_TEXT);
            write_id(out, *id);
        }
        DriverDomCommand::InsertBefore {
            parent,
            child,
            ref_id,
        } => {
            out.push(tag::INSERT_BEFORE);
            write_id(out, *parent);
            write_id(out, *child);
            write_varint(out, ref_id.map_or(NO_ID, |id| id.to_u64()));
        }
        DriverDomCommand::InsertCss { selector, value } => {
            out.push(tag::INSERT_CSS);
            match selector {
                Some(selector) => {
                    out.push(1);
                    write_str(out, selector);
                }
                None => out.push(0),
            }
            write_str(out, value);
        }
        DriverDomCommand::CreateComment { id, value } => {
            out.push(tag::CREATE_COMMENT);
            write_id(out, *id);
            write_str(out, value);
        }
        DriverDomCommand::RemoveComment { id } => {
            out.push(tag::REMOVE_COMMENT);
            write_id(out, *id);
        }
        DriverDomCommand::CallbackAdd {
            id,
            event_name,
            callback_id,
        } => {
            out.push(tag::CALLBACK_ADD);
            write_id(out, *id);
            write_str(out, event_name);
            write_varint(out, callback_id.as_u64());
        }
        DriverDomCommand::CallbackRemove {
            id,
            event_name,
            callback_id,
        } => {
            out.push(tag::CALLBACK_REMOVE);
            write_id(out, *id);
            write_str(out, event_name);
            write_varint(out, callback_id.as_u64());
        }
    }
}

fn write_id(out: &mut Vec<u8>, id: DomId) {
    debug_assert_ne!(
        id.to_u64(),
        NO_ID,
        "id zero is the `no reference node` sentinel and must not be a real DomId"
    );
    write_varint(out, id.to_u64());
}

fn write_str(out: &mut Vec<u8>, value: &str) {
    write_varint(out, value.len() as u64);
    out.extend_from_slice(value.as_bytes());
}

/// LEB128. Ids are `u64` and truncating them to `u32` would silently address the wrong node
/// once a long-lived page passed the boundary; this keeps the full range and still spends one
/// byte on the ids a real page actually has.
fn write_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;

        if value == 0 {
            out.push(byte);
            return;
        }

        out.push(byte | 0x80);
    }
}

// --- decoding ---------------------------------------------------------------------------

/// Decode a batch back into commands.
///
/// Used by server-side rendering, which replays the commands into HTML, and by the tests. In
/// the browser the equivalent decoder is the TypeScript one, which dispatches straight off
/// the tag byte without building anything.
pub fn decode_dom_commands(bytes: &[u8]) -> Result<Vec<DriverDomCommand>, String> {
    let mut cursor = Cursor::new(bytes);

    let name_count = cursor.varint()? as usize;
    let mut names: Vec<StaticString> = Vec::with_capacity(name_count);
    for _ in 0..name_count {
        names.push(StaticString::from(cursor.string()?));
    }

    let mut commands = Vec::new();
    while !cursor.is_empty() {
        commands.push(read_command(&mut cursor, &names)?);
    }

    Ok(commands)
}

fn read_command(
    cursor: &mut Cursor<'_>,
    names: &[StaticString],
) -> Result<DriverDomCommand, String> {
    let tag = cursor.byte()?;

    let command = match tag {
        tag::CREATE_NODE => DriverDomCommand::CreateNode {
            id: cursor.id()?,
            name: cursor.name(names)?,
        },
        tag::CREATE_TEXT => DriverDomCommand::CreateText {
            id: cursor.id()?,
            value: cursor.string()?,
        },
        tag::UPDATE_TEXT => DriverDomCommand::UpdateText {
            id: cursor.id()?,
            value: cursor.string()?,
        },
        tag::SET_ATTR => DriverDomCommand::SetAttr {
            id: cursor.id()?,
            name: cursor.name(names)?,
            value: cursor.string()?,
        },
        tag::REMOVE_ATTR => DriverDomCommand::RemoveAttr {
            id: cursor.id()?,
            name: cursor.name(names)?,
        },
        tag::REMOVE_NODE => DriverDomCommand::RemoveNode { id: cursor.id()? },
        tag::REMOVE_TEXT => DriverDomCommand::RemoveText { id: cursor.id()? },
        tag::INSERT_BEFORE => DriverDomCommand::InsertBefore {
            parent: cursor.id()?,
            child: cursor.id()?,
            ref_id: match cursor.varint()? {
                NO_ID => None,
                id => Some(DomId::from_u64(id)),
            },
        },
        tag::INSERT_CSS => DriverDomCommand::InsertCss {
            selector: match cursor.byte()? {
                0 => None,
                _ => Some(cursor.string()?),
            },
            value: cursor.string()?,
        },
        tag::CREATE_COMMENT => DriverDomCommand::CreateComment {
            id: cursor.id()?,
            value: cursor.string()?,
        },
        tag::REMOVE_COMMENT => DriverDomCommand::RemoveComment { id: cursor.id()? },
        tag::CALLBACK_ADD => DriverDomCommand::CallbackAdd {
            id: cursor.id()?,
            event_name: cursor.string()?,
            callback_id: CallbackId::from_u64(cursor.varint()?),
        },
        tag::CALLBACK_REMOVE => DriverDomCommand::CallbackRemove {
            id: cursor.id()?,
            event_name: cursor.string()?,
            callback_id: CallbackId::from_u64(cursor.varint()?),
        },
        other => return Err(format!("dom command: unknown tag {other}")),
    };

    Ok(command)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn is_empty(&self) -> bool {
        self.at >= self.bytes.len()
    }

    fn byte(&mut self) -> Result<u8, String> {
        let byte = self
            .bytes
            .get(self.at)
            .copied()
            .ok_or_else(|| "dom command: buffer ended early".to_string())?;
        self.at += 1;
        Ok(byte)
    }

    fn varint(&mut self) -> Result<u64, String> {
        let mut value: u64 = 0;

        for shift in (0..64).step_by(7) {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;

            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }

        Err("dom command: varint longer than 64 bits".to_string())
    }

    fn id(&mut self) -> Result<DomId, String> {
        Ok(DomId::from_u64(self.varint()?))
    }

    fn string(&mut self) -> Result<String, String> {
        let length = self.varint()? as usize;
        let end = self
            .at
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| "dom command: string runs past the end of the buffer".to_string())?;

        let text = std::str::from_utf8(&self.bytes[self.at..end])
            .map_err(|err| format!("dom command: string is not utf-8: {err}"))?
            .to_string();

        self.at = end;
        Ok(text)
    }

    fn name(&mut self, names: &[StaticString]) -> Result<StaticString, String> {
        let position = self.varint()? as usize;
        names
            .get(position)
            .cloned()
            .ok_or_else(|| format!("dom command: name index {position} is not in the dictionary"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u64) -> DomId {
        DomId::from_u64(value)
    }

    /// `unwrap`/`expect` are denied workspace-wide, and a decode failure in a test wants
    /// to say what went wrong anyway.
    fn decoded(bytes: &[u8]) -> Vec<DriverDomCommand> {
        match decode_dom_commands(bytes) {
            Ok(commands) => commands,
            Err(err) => panic!("decode failed: {err}"),
        }
    }

    /// Every variant, through the encoder and back. A field written in the wrong order
    /// decodes as garbage rather than failing, so this - not a reading of the code - is what
    /// says the two halves agree.
    /// Shared with `dom_wire.test.ts` - see `matches_the_cross_language_fixture`.
    fn fixture_commands() -> Vec<DriverDomCommand> {
        vec![
            DriverDomCommand::CreateNode {
                id: id(4),
                name: "div".into(),
            },
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "class".into(),
                value: "row".to_string(),
            },
            DriverDomCommand::CreateNode {
                id: id(300),
                name: "div".into(),
            },
            DriverDomCommand::RemoveAttr {
                id: id(300),
                name: "class".into(),
            },
            DriverDomCommand::CreateText {
                id: id(5),
                value: "zażółć 🦀".to_string(),
            },
            DriverDomCommand::UpdateText {
                id: id(5),
                value: String::new(),
            },
            DriverDomCommand::InsertBefore {
                parent: id(1),
                child: id(4),
                ref_id: Some(id(70000)),
            },
            DriverDomCommand::InsertBefore {
                parent: id(1),
                child: id(4),
                ref_id: None,
            },
            DriverDomCommand::InsertCss {
                selector: Some(".a".to_string()),
                value: "color:red".to_string(),
            },
            DriverDomCommand::InsertCss {
                selector: None,
                value: "@media print{}".to_string(),
            },
            DriverDomCommand::CreateComment {
                id: id(6),
                value: "row".to_string(),
            },
            DriverDomCommand::RemoveComment { id: id(6) },
            DriverDomCommand::RemoveNode { id: id(4) },
            DriverDomCommand::RemoveText { id: id(5) },
            DriverDomCommand::CallbackAdd {
                id: id(4),
                event_name: "click".to_string(),
                callback_id: CallbackId::from_u64(77),
            },
            DriverDomCommand::CallbackRemove {
                id: id(4),
                event_name: "click".to_string(),
                callback_id: CallbackId::from_u64(300),
            },
        ]
    }

    /// The same bytes are asserted from the other side in
    /// `driver_module/src_js/api/command/dom/dom_wire.test.ts`.
    ///
    /// Round-tripping through this file only proves the encoder and decoder here agree with
    /// each other; they could agree on something the browser reads differently. This is the
    /// pin between the two languages: change the format and one of the two tests fails,
    /// whichever side you changed.
    const FIXTURE: &[u8] = &[
        2, 3, 100, 105, 118, 5, 99, 108, 97, 115, 115, 1, 4, 0, 4, 4, 1, 3, 114, 111, 119, 1, 172,
        2, 0, 5, 172, 2, 1, 2, 5, 15, 122, 97, 197, 188, 195, 179, 197, 130, 196, 135, 32, 240,
        159, 166, 128, 3, 5, 0, 8, 1, 4, 240, 162, 4, 8, 1, 4, 0, 9, 1, 2, 46, 97, 9, 99, 111, 108,
        111, 114, 58, 114, 101, 100, 9, 0, 14, 64, 109, 101, 100, 105, 97, 32, 112, 114, 105, 110,
        116, 123, 125, 10, 6, 3, 114, 111, 119, 11, 6, 6, 4, 7, 5, 12, 4, 5, 99, 108, 105, 99, 107,
        77, 13, 4, 5, 99, 108, 105, 99, 107, 172, 2,
    ];

    #[test]
    fn matches_the_cross_language_fixture() {
        assert_eq!(
            encode_dom_commands(&fixture_commands()),
            FIXTURE,
            "the wire format changed - update dom_wire.test.ts to match, or this is a bug"
        );
    }

    #[test]
    fn round_trips_every_variant() {
        let commands = vec![
            DriverDomCommand::CreateNode {
                id: id(4),
                name: "div".into(),
            },
            DriverDomCommand::CreateText {
                id: id(5),
                value: "hello".to_string(),
            },
            DriverDomCommand::UpdateText {
                id: id(5),
                value: String::new(),
            },
            DriverDomCommand::SetAttr {
                id: id(4),
                name: "class".into(),
                value: "col-md-1".to_string(),
            },
            DriverDomCommand::RemoveAttr {
                id: id(4),
                name: "class".into(),
            },
            DriverDomCommand::RemoveNode { id: id(4) },
            DriverDomCommand::RemoveText { id: id(5) },
            DriverDomCommand::InsertBefore {
                parent: id(1),
                child: id(4),
                ref_id: Some(id(9)),
            },
            DriverDomCommand::InsertBefore {
                parent: id(1),
                child: id(4),
                ref_id: None,
            },
            DriverDomCommand::InsertCss {
                selector: Some(".a".to_string()),
                value: "color: red".to_string(),
            },
            DriverDomCommand::InsertCss {
                selector: None,
                value: "@media print {}".to_string(),
            },
            DriverDomCommand::CreateComment {
                id: id(6),
                value: "row".to_string(),
            },
            DriverDomCommand::RemoveComment { id: id(6) },
            DriverDomCommand::CallbackAdd {
                id: id(4),
                event_name: "click".to_string(),
                callback_id: CallbackId::from_u64(77),
            },
            DriverDomCommand::CallbackRemove {
                id: id(4),
                event_name: "click".to_string(),
                callback_id: CallbackId::from_u64(77),
            },
        ];

        let decoded = decoded(&encode_dom_commands(&commands));

        assert_eq!(format!("{decoded:?}"), format!("{commands:?}"));
    }

    /// Text is user data and may be anything; the length prefix counts bytes, not characters.
    #[test]
    fn round_trips_non_ascii_and_empty_text() {
        let commands = vec![
            DriverDomCommand::CreateText {
                id: id(4),
                value: "zażółć gęślą jaźń - 日本語 - 🦀".to_string(),
            },
            DriverDomCommand::CreateText {
                id: id(5),
                value: String::new(),
            },
        ];

        let decoded = decoded(&encode_dom_commands(&commands));
        assert_eq!(format!("{decoded:?}"), format!("{commands:?}"));
    }

    /// The whole point of the dictionary: a name repeated across rows is stored once and
    /// referenced by a one-byte index thereafter.
    #[test]
    fn a_repeated_name_is_stored_once() {
        let rows = 500;
        let commands: Vec<DriverDomCommand> = (0..rows)
            .map(|index| DriverDomCommand::CreateNode {
                id: id(index + 4),
                name: "td".into(),
            })
            .collect();

        let encoded = encode_dom_commands(&commands);

        // dictionary: one entry, length 2, "td"
        assert_eq!(&encoded[0..4], &[1, 2, b't', b'd']);

        // Each command is then tag + id + name index and nothing else, so the name is paid
        // for once however many rows there are. Ids here need two varint bytes past 127.
        let body = encoded.len() - 4;
        assert!(
            body <= rows as usize * 4,
            "{body} bytes for {rows} rows - the name is being repeated"
        );

        let decoded = decoded(&encoded);
        assert_eq!(decoded.len() as u64, rows);
        assert!(matches!(
            &decoded[0],
            DriverDomCommand::CreateNode { name, .. } if name.as_str() == "td"
        ));
    }

    /// Ids are `u64` and must survive the whole range, not just the small values a page
    /// actually reaches.
    #[test]
    fn round_trips_large_ids() {
        let commands = vec![DriverDomCommand::InsertBefore {
            parent: id(u64::MAX),
            child: id(u64::MAX - 1),
            ref_id: Some(id(1 << 40)),
        }];

        let decoded = decoded(&encode_dom_commands(&commands));
        assert_eq!(format!("{decoded:?}"), format!("{commands:?}"));
    }

    #[test]
    fn an_empty_batch_round_trips() {
        let decoded = decoded(&encode_dom_commands(&[]));
        assert!(decoded.is_empty());
    }

    /// A corrupt buffer has to be reported, not panic the runtime.
    #[test]
    fn malformed_input_is_an_error() {
        assert!(decode_dom_commands(&[0, 200]).is_err(), "unknown tag");
        assert!(
            decode_dom_commands(&[0, tag::CREATE_TEXT, 4, 9]).is_err(),
            "string longer than the buffer"
        );
        assert!(
            decode_dom_commands(&[0, tag::CREATE_NODE, 4, 0]).is_err(),
            "name index with an empty dictionary"
        );
        assert!(
            decode_dom_commands(&[0, tag::CREATE_NODE]).is_err(),
            "buffer ends mid-command"
        );
    }
}
