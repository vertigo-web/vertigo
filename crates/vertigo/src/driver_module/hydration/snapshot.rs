use vertigo_macro::AutoJsJson;

/// Browser DOM state read in pre-order traversal.
///
/// The index into [`Self::nodes`] is the node address in the protocol:
/// [`NodeAdopt`](crate::dev::command::DriverDomCommand::NodeAdopt) and
/// [`SnapshotRemove`](crate::dev::command::DriverDomCommand::SnapshotRemove) commands refer
/// to the actual node that JS holds in an array built during the same traversal.
/// Element 0 is `<html>`.
#[derive(AutoJsJson, Debug, Clone)]
pub(crate) struct DomSnapshot {
    pub nodes: Vec<SnapshotNode>,
    pub head: Option<u32>,
    pub body: Option<u32>,
}

#[derive(AutoJsJson, Debug, Clone)]
pub(crate) enum SnapshotNode {
    Element {
        /// The `tagName` lowercased. Rust-side matching compares case-insensitively,
        /// which is correct for both HTML and SVG, and avoids shipping the `SVG_TAGS`
        /// array to wasm.
        name: String,
        attrs: Vec<SnapshotAttr>,
        children: Vec<u32>,
    },
    Text {
        value: String,
    },
    Comment {
        value: String,
    },
}

#[derive(AutoJsJson, Debug, Clone)]
pub(crate) struct SnapshotAttr {
    pub name: String,
    pub value: String,
}

impl DomSnapshot {
    /// Returns the node at the given index, or `None` if the index is out of range.
    pub fn node(&self, index: u32) -> Option<&SnapshotNode> {
        self.nodes.get(index as usize)
    }

    pub fn children(&self, index: u32) -> &[u32] {
        match self.node(index) {
            Some(SnapshotNode::Element { children, .. }) => children.as_slice(),
            _ => &[],
        }
    }

    pub fn attrs(&self, index: u32) -> &[SnapshotAttr] {
        match self.node(index) {
            Some(SnapshotNode::Element { attrs, .. }) => attrs.as_slice(),
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsJsonSerialize, dev::command::decode_json};

    fn sample() -> DomSnapshot {
        DomSnapshot {
            nodes: vec![
                SnapshotNode::Element {
                    name: "html".to_string(),
                    attrs: vec![SnapshotAttr {
                        name: "lang".to_string(),
                        value: "pl".to_string(),
                    }],
                    children: vec![1, 2],
                },
                SnapshotNode::Element {
                    name: "head".to_string(),
                    attrs: vec![],
                    children: vec![],
                },
                SnapshotNode::Element {
                    name: "body".to_string(),
                    attrs: vec![],
                    children: vec![3, 4],
                },
                SnapshotNode::Text {
                    value: "zażółć 🦀".to_string(),
                },
                SnapshotNode::Comment {
                    value: "a marker".to_string(),
                },
            ],
            head: Some(1),
            body: Some(2),
        }
    }

    #[test]
    fn a_snapshot_survives_the_json_round_trip() {
        let json = sample().to_json();

        let back = match decode_json::<DomSnapshot>(json) {
            Ok(value) => value,
            Err(err) => panic!("decode failed: {err}"),
        };

        assert_eq!(format!("{back:?}"), format!("{:?}", sample()));
    }

    #[test]
    fn children_of_a_non_element_is_empty() {
        let snapshot = sample();

        assert_eq!(snapshot.children(0), &[1, 2]);
        assert_eq!(snapshot.children(3), &[] as &[u32]);
        assert_eq!(snapshot.children(999), &[] as &[u32]);
    }
}
