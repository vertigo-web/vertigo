use crate::DomId;
use crate::driver_module::utils::json::JsonMapBuilder;
use crate::driver_module::callbacks::CallbackId;

#[derive(Clone, Debug)]
pub enum DriverDomCommand {
    MountNode {
        id: DomId,
    },
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

    pub fn into_string(self) -> String {
        let mut out = JsonMapBuilder::new();

        match self {
            Self::MountNode { id } => {
                out.set_string("type", "mount_node");
                out.set_u64("id", id.to_u64());
            }
            Self::CreateNode { id, name } => {
                out.set_string("type", "create_node");
                out.set_u64("id", id.to_u64());
                out.set_string("name", name);
            }
            Self::CreateText { id, value } => {
                out.set_string("type", "create_text");
                out.set_u64("id", id.to_u64());
                out.set_string("value", value.as_str());
            }
            Self::UpdateText { id, value } => {
                out.set_string("type", "update_text");
                out.set_u64("id", id.to_u64());
                out.set_string("value", value.as_str());
            }
            Self::SetAttr { id, name, value } => {
                out.set_string("type", "set_attr");
                out.set_u64("id", id.to_u64());
                out.set_string("name", name);
                out.set_string("value", value.as_str());
            }
            Self::RemoveNode { id } => {
                out.set_string("type", "remove_node");
                out.set_u64("id", id.to_u64());
            }
            Self::RemoveText { id } => {
                out.set_string("type", "remove_text");
                out.set_u64("id", id.to_u64());
            }

            Self::CreateComment { id, value } => {
                out.set_string("type", "create_comment");
                out.set_u64("id", id.to_u64());
                out.set_string("value", value.as_str());
            },
            Self::RemoveComment { id } => {
                out.set_string("type", "remove_comment");
                out.set_u64("id", id.to_u64());
            },

            Self::InsertBefore { parent, child, ref_id } => {
                out.set_string("type", "insert_before");
                out.set_u64("parent", parent.to_u64());
                out.set_u64("child", child.to_u64());

                match ref_id {
                    Some(ref_id) => {
                        out.set_u64("ref_id", ref_id.to_u64());
                    }
                    None => {
                        out.set_null("ref_id");
                    }
                }
            }
            Self::InsertCss { selector, value } => {
                out.set_string("type", "insert_css");
                out.set_string("selector", selector.as_str());
                out.set_string("value", value.as_str());
            },
            Self::CallbackAdd { id, event_name, callback_id } => {
                out.set_string("type", "callback_add");
                out.set_u64("id", id.to_u64());
                out.set_string("event_name", event_name.as_str());
                out.set_u64("callback_id", callback_id.as_u64());
            },
            Self::CallbackRemove { id, event_name, callback_id } => {
                out.set_string("type", "callback_remove");
                out.set_u64("id", id.to_u64());
                out.set_string("event_name", event_name.as_str());
                out.set_u64("callback_id", callback_id.as_u64());
            }
        }

        out.build()
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
