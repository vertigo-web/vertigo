use vertigo::RealDomId;
use crate::utils::json::JsonMapBuilder;

pub enum DriverDomCommand {
    MountNode {
        id: RealDomId,
    },
    CreateNode {
        id: RealDomId,
        name: &'static str
    },
    RenameNode {
        id: RealDomId,
        new_name: &'static str
    },
    CreateText {
        id: RealDomId,
        value: String
    },
    UpdateText {
        id: RealDomId,
        value: String
    },
    SetAttr {
        id: RealDomId,
        key: &'static str,
        value: String
    },
    RemoveAttr{
        id: RealDomId,
        name: &'static str
    },
    RemoveNode {
        id: RealDomId
    },
    RemoveText {
        id: RealDomId,
    },
    InsertBefore {
        parent: RealDomId,
        child: RealDomId,
        ref_id: Option<RealDomId>
    },
    InsertCss {
        selector: String,
        value: String
    }
}

impl DriverDomCommand {
    pub fn into_string(self) -> String {
        let mut out = JsonMapBuilder::new();

        match self {
            Self::MountNode { id } => {
                out.set_string("type", "mount_node");
                out.set_u64("id", id.to_u64());
            },
            Self::CreateNode { id, name } => {
                out.set_string("type", "create_node");
                out.set_u64("id", id.to_u64());
                out.set_string("name", name);
            },
            Self::RenameNode { id, new_name } => {
                out.set_string("type", "rename_node");
                out.set_u64("id", id.to_u64());
                out.set_string("new_name", new_name);
            },
            Self::CreateText { id, value } => {
                out.set_string("type", "create_text");
                out.set_u64("id", id.to_u64());
                out.set_string("value", value.as_str());
            },
            Self::UpdateText {id, value } => {
                out.set_string("type", "update_text");
                out.set_u64("id", id.to_u64());
                out.set_string("value", value.as_str());
            },
            Self::SetAttr { id, key, value } => {
                out.set_string("type", "set_attr");
                out.set_u64("id", id.to_u64());
                out.set_string("key", key);
                out.set_string("value", value.as_str());
            },
            Self::RemoveAttr { id, name } => {
                out.set_string("type", "remove_attr");
                out.set_u64("id", id.to_u64());
                out.set_string("name", name);
            },
            Self::RemoveNode { id } => {
                out.set_string("type", "remove_node");
                out.set_u64("id", id.to_u64());
            },
            Self::RemoveText { id } => {
                out.set_string("type", "remove_text");
                out.set_u64("id", id.to_u64());
            },
            Self::InsertBefore { parent, child, ref_id } => {
                out.set_string("type", "insert_before");
                out.set_u64("parent", parent.to_u64());
                out.set_u64("child", child.to_u64());

                match ref_id {
                    Some(ref_id) => {
                        out.set_u64("ref_id", ref_id.to_u64());
                    },
                    None => {
                        out.set_null("ref_id");
                    }
                }
            },
            Self::InsertCss { selector, value } => {
                out.set_string("type", "insert_css");
                out.set_string("selector", selector.as_str());
                out.set_string("value", value.as_str());
            }
        }

        out.build()
    }
}