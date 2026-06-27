//! Server (WebSocket) messages, tagged by variant name, e.g.:
//! `{ "init": { ... } }`, `{ "set": { "query_id", "model_id", "model" } }`,
//! `{ "delete": { "query_id", "model_id" } }`.
//! The `list` and `model` fields are raw `JsJson` (flexible shape from the backend).
//!
//! From [`crate::JsJson`] (e.g. from [`crate::WebsocketMessage::Message`]): [`WsServerMessageFrom::from_js_json`].
//!
//! ## Protocol source of truth
//!
//! Mirrors the server's data-package enum: the server emits one of `init` / `set` /
//! `delete` / `batch` / `message` per frame, with `query_id` identifying the
//! subscription. When the server adds or renames a variant, sync this enum and
//! `from_js_json` will report the unknown tag by name instead of failing silently.

use crate::{AutoJsJson, JsJson};

use super::types::WebsocketQueryId;

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub struct WsInitDataModel {
    pub id: String,
    pub model: JsJson,
}

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub struct WsInitData {
    pub query_id: WebsocketQueryId,
    pub list: Vec<WsInitDataModel>,
}

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub struct WsSetData {
    pub query_id: WebsocketQueryId,
    pub model_id: String,
    pub model: JsJson,
}

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub struct WsDeleteData {
    pub query_id: WebsocketQueryId,
    pub model_id: String,
}

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub enum WsMessageKind {
    Log,
    Error,
}

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
#[js_json(rename_all = "camelCase")]
pub struct WsMessageData {
    pub query_id: WebsocketQueryId,
    pub kind: WsMessageKind,
    pub message: String,
}

#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub enum WsServerMessageFrom {
    #[js_json(rename = "init")]
    Init(WsInitData),
    #[js_json(rename = "set")]
    Set(WsSetData),
    #[js_json(rename = "delete")]
    Delete(WsDeleteData),
    #[js_json(rename = "batch")]
    Batch(Vec<WsServerMessageFrom>),
    #[js_json(rename = "message")]
    Message(WsMessageData),
}

/// Inspects the top-level shape of a server message and returns the variant tag (the
/// single object key) when present — used purely for diagnostics on parse failure.
fn unknown_tag(json: &JsJson) -> Option<String> {
    match json {
        JsJson::Object(map) if map.len() == 1 => map.keys().next().cloned(),
        _ => None,
    }
}

impl WsServerMessageFrom {
    /// Decodes a message already as `JsJson` (as delivered by the WebSocket driver).
    ///
    /// On parse failure, enriches the error with the top-level tag the server used
    /// (e.g. `initOne`) so log readers can tell *which* variant is missing rather
    /// than just "value did not match any variant".
    pub(crate) fn from_js_json(json: JsJson) -> Result<Self, String> {
        crate::from_json::<Self>(json.clone()).map_err(|err| match unknown_tag(&json) {
            Some(tag) => format!("unknown variant `{tag}` (server/client protocol drift): {err}"),
            None => err,
        })
    }

    /// `Batch` has no single id (it carries multiple sub-messages); all other variants do.
    pub(crate) fn query_id(&self) -> Option<WebsocketQueryId> {
        match self {
            WsServerMessageFrom::Init(d) => Some(d.query_id.clone()),
            WsServerMessageFrom::Set(d) => Some(d.query_id.clone()),
            WsServerMessageFrom::Delete(d) => Some(d.query_id.clone()),
            WsServerMessageFrom::Message(d) => Some(d.query_id.clone()),
            WsServerMessageFrom::Batch(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber, to_json};

    use super::{super::types::WebsocketQueryId, *};

    fn ctx() -> JsJsonContext {
        JsJsonContext::new("")
    }

    fn sample_model_js() -> JsJson {
        JsJson::Object(BTreeMap::from([
            ("name".into(), JsJson::String("Test".into())),
            ("ref_id".into(), JsJson::String("1".into())),
            ("id".into(), JsJson::Number(JsJsonNumber(1.0))),
            ("flag".into(), JsJson::False),
            ("owner_id".into(), JsJson::Number(JsJsonNumber(1.0))),
        ]))
    }

    #[test]
    fn ws_server_message_init_round_trips() {
        let msg = WsServerMessageFrom::Init(WsInitData {
            query_id: WebsocketQueryId("q1".into()),
            list: vec![WsInitDataModel {
                id: "1".into(),
                model: sample_model_js(),
            }],
        });
        let json = to_json(msg.clone());
        let back = crate::from_json::<WsServerMessageFrom>(json).expect("from_json");
        assert_eq!(back, msg);
    }

    #[test]
    fn ws_server_message_set_round_trips() {
        let msg = WsServerMessageFrom::Set(WsSetData {
            query_id: WebsocketQueryId("q1".into()),
            model_id: "6".into(),
            model: sample_model_js(),
        });
        let json = to_json(msg.clone());
        let back = crate::from_json::<WsServerMessageFrom>(json).expect("from_json");
        assert_eq!(back, msg);
    }

    #[test]
    fn ws_server_message_delete_round_trips() {
        let msg = WsServerMessageFrom::Delete(WsDeleteData {
            query_id: WebsocketQueryId("query_0".into()),
            model_id: "7".into(),
        });
        let json = to_json(msg.clone());
        let back = crate::from_json::<WsServerMessageFrom>(json).expect("from_json");
        assert_eq!(back, msg);
    }

    #[test]
    fn from_js_json_reports_unknown_variant_by_tag() {
        // Simulates server drift (e.g. a new `InitOne` variant) — the parser must report
        // the offending tag in the error so logs are actionable, and must not panic.
        let raw = JsJson::Object(BTreeMap::from([(
            "initOne".into(),
            JsJson::Object(BTreeMap::from([
                ("queryId".into(), JsJson::String("query_0".into())),
                ("modelId".into(), JsJson::String("1".into())),
                ("model".into(), JsJson::Number(JsJsonNumber(1.0))),
            ])),
        )]));

        let err = WsServerMessageFrom::from_js_json(raw).expect_err("should not decode");
        assert!(
            err.contains("`initOne`"),
            "error must name the unknown tag, got: {err}"
        );
    }

    #[test]
    fn ws_server_message_init_matches_from_json_on_jsjson() {
        let msg = WsServerMessageFrom::Init(WsInitData {
            query_id: WebsocketQueryId("query_0".into()),
            list: vec![WsInitDataModel {
                id: "1".into(),
                model: JsJson::Number(JsJsonNumber(1.0)),
            }],
        });
        let json = to_json(msg.clone());
        let from_public = crate::from_json::<WsServerMessageFrom>(json.clone()).expect("from_json");
        let from_trait = WsServerMessageFrom::from_json(ctx(), json).expect("from_json trait");
        assert_eq!(from_public, msg);
        assert_eq!(from_trait, msg);
    }
}
