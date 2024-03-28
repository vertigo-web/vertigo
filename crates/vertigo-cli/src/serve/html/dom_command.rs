use serde::{Deserialize, Serialize};
use vertigo::{from_json, JsJson, JsJsonContext, JsJsonDeserialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DomCommand {
    #[serde(rename = "create_node")]
    CreateNode { id: u64, name: String },
    #[serde(rename = "create_text")]
    CreateText { id: u64, value: String },
    #[serde(rename = "update_text")]
    UpdateText { id: u64, value: String },
    #[serde(rename = "set_attr")]
    SetAttr {
        id: u64,
        name: String,
        value: String,
    },
    #[serde(rename = "remove_node")]
    RemoveNode { id: u64 },
    #[serde(rename = "remove_text")]
    RemoveText { id: u64 },
    #[serde(rename = "insert_before")]
    InsertBefore {
        parent: u64,
        child: u64,
        ref_id: Option<u64>,
    },
    #[serde(rename = "insert_css")]
    InsertCss { selector: String, value: String },
    #[serde(rename = "create_comment")]
    CreateComment { id: u64, value: String },
    #[serde(rename = "remove_comment")]
    RemoveComment { id: u64 },
    #[serde(rename = "callback_add")]
    CallbackAdd {
        id: u64,
        event_name: String,
        callback_id: u64,
    },
    #[serde(rename = "callback_remove")]
    CallbackRemove {
        id: u64,
        event_name: String,
        callback_id: u64,
    },
}

impl JsJsonDeserialize for DomCommand {
    fn from_json(context: JsJsonContext, mut json: JsJson) -> Result<Self, JsJsonContext> {
        let type_param: String = json.get_property(&context, "type")?;

        let result = match type_param.as_str() {
            "create_node" => Self::CreateNode {
                id: json.get_property(&context, "id")?,
                name: json.get_property(&context, "name")?,
            },
            "create_text" => Self::CreateText {
                id: json.get_property(&context, "id")?,
                value: json.get_property(&context, "value")?,
            },
            "update_text" => Self::UpdateText {
                id: json.get_property(&context, "id")?,
                value: json.get_property(&context, "value")?,
            },
            "set_attr" => Self::SetAttr {
                id: json.get_property(&context, "id")?,
                name: json.get_property(&context, "name")?,
                value: json.get_property(&context, "value")?,
            },
            "remove_node" => Self::RemoveNode {
                id: json.get_property(&context, "id")?,
            },
            "remove_text" => Self::RemoveText {
                id: json.get_property(&context, "id")?,
            },
            "insert_before" => Self::InsertBefore {
                parent: json.get_property(&context, "parent")?,
                child: json.get_property(&context, "child")?,
                ref_id: json.get_property(&context, "ref_id")?,
            },
            "insert_css" => Self::InsertCss {
                selector: json.get_property(&context, "selector")?,
                value: json.get_property(&context, "value")?,
            },
            "create_comment" => Self::CreateComment {
                id: json.get_property(&context, "id")?,
                value: json.get_property(&context, "value")?,
            },
            "remove_comment" => Self::RemoveComment {
                id: json.get_property(&context, "id")?,
            },
            "callback_add" => Self::CallbackAdd {
                id: json.get_property(&context, "id")?,
                event_name: json.get_property(&context, "event_name")?,
                callback_id: json.get_property(&context, "callback_id")?,
            },
            "callback_remove" => Self::CallbackRemove {
                id: json.get_property(&context, "id")?,
                event_name: json.get_property(&context, "event_name")?,
                callback_id: json.get_property(&context, "callback_id")?,
            },
            _ => {
                unreachable!();
            }
        };

        Ok(result)
    }
}

pub fn dom_command_from_js_json(json: JsJson) -> Result<Vec<DomCommand>, String> {
    from_json::<Vec<DomCommand>>(json)
}
