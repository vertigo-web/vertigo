use crate::dev::DomId;

use crate::driver_module::utils::json::JsonMapBuilder;

pub enum DriverDomCommand {
    MountNode {
        id: DomId,
    },
    CreateNode {
        id: DomId,
        name: &'static str,
    },
    RenameNode {
        id: DomId,
        new_name: &'static str,
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
    RemoveAttr {
        id: DomId,
        name: &'static str,
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
    UpdateComment{
        id: DomId,
        value: String,
    },
    RemoveComment {
        id: DomId,
    },
}

impl DriverDomCommand {
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
            Self::RenameNode { id, new_name } => {
                out.set_string("type", "rename_node");
                out.set_u64("id", id.to_u64());
                out.set_string("new_name", new_name);
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
            Self::RemoveAttr { id, name } => {
                out.set_string("type", "remove_attr");
                out.set_u64("id", id.to_u64());
                out.set_string("name", name);
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
            Self::UpdateComment { id, value } => {
                out.set_string("type", "update_comment");
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
            }
        }

        out.build()
    }
}
