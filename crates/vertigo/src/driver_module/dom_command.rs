use crate::{DomId, JsJson, JsJsonObjectBuilder};

use super::api::CallbackId;

#[derive(Clone, Debug)]
pub enum DriverDomCommand {
    CreateNode {
        id: DomId,
        name: &'static str,
    },
    CreateText {
        id: DomId,
        value: String,
    },
    UpdateText {
        id: DomId,
        value: String,
    },
    SetAttr {
        id: DomId,
        name: &'static str,
        value: String,
    },
    RemoveNode {
        id: DomId,
    },
    RemoveText {
        id: DomId,
    },
    InsertBefore {
        parent: DomId,
        child: DomId,
        ref_id: Option<DomId>,
    },
    InsertCss {
        selector: String,
        value: String,
    },

    CreateComment {
        id: DomId,
        value: String,
    },
    RemoveComment {
        id: DomId,
    },
    CallbackAdd {
        id: DomId,
        event_name: String,
        callback_id: CallbackId,
    },
    CallbackRemove {
        id: DomId,
        event_name: String,
        callback_id: CallbackId,
    }
}

impl DriverDomCommand {
    fn is_event(&self) -> bool {
        matches!(self,
            Self::RemoveNode { .. } |
            Self::RemoveText { .. } |
            Self::RemoveComment { .. }
        )
    }

    pub fn into_string(self) -> JsJson {
        match self {
            Self::CreateNode { id, name } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "create_node")
                    .insert("id", id.to_u64())
                    .insert("name", name)
                    .get()
            }
            Self::CreateText { id, value } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "create_text")
                    .insert("id", id.to_u64())
                    .insert("value", value)
                    .get()
            }
            Self::UpdateText { id, value } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "update_text")
                    .insert("id", id.to_u64())
                    .insert("value", value)
                    .get()
            }
            Self::SetAttr { id, name, value } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "set_attr")
                    .insert("id", id.to_u64())
                    .insert("name", name)
                    .insert("value", value)
                    .get()
            }
            Self::RemoveNode { id } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "remove_node")
                    .insert("id", id.to_u64())
                    .get()
            }
            Self::RemoveText { id } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "remove_text")
                    .insert("id", id.to_u64())
                    .get()
            }

            Self::CreateComment { id, value } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "create_comment")
                    .insert("id", id.to_u64())
                    .insert("value", value)
                    .get()
            },
            Self::RemoveComment { id } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "remove_comment")
                    .insert("id", id.to_u64())
                    .get()
            },

            Self::InsertBefore { parent, child, ref_id } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "insert_before")
                    .insert("parent", parent.to_u64())
                    .insert("child", child.to_u64())
                    .insert("ref_id", ref_id.map(|value| value.to_u64()))
                    .get()
            }
            Self::InsertCss { selector, value } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "insert_css")
                    .insert("selector", selector)
                    .insert("value", value)
                    .get()
            },
            Self::CallbackAdd { id, event_name, callback_id } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "callback_add")
                    .insert("id", id.to_u64())
                    .insert("event_name", event_name)
                    .insert("callback_id", callback_id.as_u64())
                    .get()
            },
            Self::CallbackRemove { id, event_name, callback_id } => {
                JsJsonObjectBuilder::default()
                    .insert("type", "callback_remove")
                    .insert("id", id.to_u64())
                    .insert("event_name", event_name)
                    .insert("callback_id", callback_id.as_u64())
                    .get()
            }
        }
    }
}

pub fn sort_commands(list: Vec<DriverDomCommand>) -> Vec<DriverDomCommand> {
    let mut dom = Vec::new();
    let mut events = Vec::new();

    for command in list {
        if command.is_event() {
            events.push(command);
        } else {
            dom.push(command);
        }
    }

    dom.extend(events.into_iter());

    dom
}
