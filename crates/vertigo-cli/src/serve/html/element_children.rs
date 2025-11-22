use std::collections::HashMap;

use vertigo::DomId;

struct ChildrenNode {
    left: DomId,
    right: DomId,
}

pub struct ElementChildren {
    first_child: Option<DomId>,
    children: HashMap<DomId, ChildrenNode>,
}

impl ElementChildren {
    pub fn new() -> ElementChildren {
        ElementChildren {
            first_child: None,
            children: HashMap::new(),
        }
    }

    pub fn get_all(&self) -> Vec<DomId> {
        if self.children.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();

        let Some(mut id_ptr) = self.first_child else {
            log::error!("Unreachable in ElementChildren::get_all (1)");
            return Vec::new();
        };

        for _ in 0..self.children.len() {
            let Some(node) = self.children.get(&id_ptr) else {
                log::error!("Unreachable in ElementChildren::get_all (2)");
                return result;
            };

            result.push(id_ptr);
            id_ptr = node.right;
        }

        // We expect to turn a full circle
        assert_eq!(Some(id_ptr), self.first_child);

        result
    }

    fn insert_on_left(&mut self, node_right_id: DomId, node_id: DomId) {
        let Some(node_right) = self.children.get_mut(&node_right_id) else {
            log::error!("Unreachable in ElementChildren::insert_on_left (1)");
            return;
        };

        let node_left_id = node_right.left;
        node_right.left = node_id;

        let Some(node_left) = self.children.get_mut(&node_left_id) else {
            log::error!("Unreachable in ElementChildren::insert_on_left (2)");
            return;
        };
        node_left.right = node_id;

        self.children.insert(
            node_id,
            ChildrenNode {
                left: node_left_id,
                right: node_right_id,
            },
        );
    }

    pub fn insert_before(&mut self, ref_id: Option<DomId>, child: DomId) {
        if ref_id == Some(child) {
            log::error!("ref_id must not be equal to child_id, ref_id={ref_id:?}, child={child:?}");
            return;
        }

        self.remove(child);

        let right_node_id = {
            if let Some(ref_id) = ref_id {
                ref_id
            } else if let Some(first_child) = self.first_child {
                first_child
            } else {
                assert_eq!(self.children.len(), 0);

                self.first_child = Some(child);
                self.children.insert(
                    child,
                    ChildrenNode {
                        left: child,
                        right: child,
                    },
                );
                return;
            }
        };

        self.insert_on_left(right_node_id, child);

        if self.first_child == ref_id {
            self.first_child = Some(child);
        }
    }

    pub fn remove(&mut self, node_id: DomId) {
        let Some(node) = self.children.remove(&node_id) else {
            return;
        };

        if node.left == node_id {
            assert_eq!(node.right, node_id);

            self.first_child = None;
            return;
        }

        let Some(node_right) = self.children.get_mut(&node.right) else {
            log::error!("Unreachable in ElementChildren::remove (1)");
            return;
        };

        node_right.left = node.left;

        let Some(node_left) = self.children.get_mut(&node.left) else {
            log::error!("Unreachable in ElementChildren::remove (2)");
            return;
        };

        node_left.right = node.right;

        if self.first_child == Some(node_id) {
            self.first_child = Some(node.right);
        }
    }
}

#[cfg(test)]
mod tests {
    use vertigo::DomId;

    use super::ElementChildren;

    #[test]
    fn test_children() {
        let mut children = ElementChildren::new();

        assert_eq!(children.first_child, None);
        assert_eq!(children.get_all(), Vec::<DomId>::new());

        children.insert_before(None, DomId::from_u64(44));

        assert_eq!(children.first_child, Some(DomId::from_u64(44)));
        assert_eq!(children.get_all(), vec!(DomId::from_u64(44)));

        children.insert_before(None, DomId::from_u64(55));

        assert_eq!(children.first_child, Some(DomId::from_u64(44)));
        assert_eq!(
            children.get_all(),
            vec!(DomId::from_u64(44), DomId::from_u64(55))
        );

        children.insert_before(None, DomId::from_u64(66));

        assert_eq!(children.first_child, Some(DomId::from_u64(44)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(44),
                DomId::from_u64(55),
                DomId::from_u64(66)
            )
        );

        children.insert_before(Some(DomId::from_u64(44)), DomId::from_u64(33));

        assert_eq!(children.first_child, Some(DomId::from_u64(33)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(33),
                DomId::from_u64(44),
                DomId::from_u64(55),
                DomId::from_u64(66)
            )
        );

        children.insert_before(Some(DomId::from_u64(44)), DomId::from_u64(35));

        assert_eq!(children.first_child, Some(DomId::from_u64(33)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(33),
                DomId::from_u64(35),
                DomId::from_u64(44),
                DomId::from_u64(55),
                DomId::from_u64(66)
            )
        );

        children.remove(DomId::from_u64(55));

        assert_eq!(children.first_child, Some(DomId::from_u64(33)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(33),
                DomId::from_u64(35),
                DomId::from_u64(44),
                DomId::from_u64(66)
            )
        );

        children.remove(DomId::from_u64(66));

        assert_eq!(children.first_child, Some(DomId::from_u64(33)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(33),
                DomId::from_u64(35),
                DomId::from_u64(44)
            )
        );

        children.remove(DomId::from_u64(33));

        assert_eq!(children.first_child, Some(DomId::from_u64(35)));
        assert_eq!(
            children.get_all(),
            vec!(DomId::from_u64(35), DomId::from_u64(44))
        );

        children.insert_before(Some(DomId::from_u64(44)), DomId::from_u64(36));

        assert_eq!(children.first_child, Some(DomId::from_u64(35)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(35),
                DomId::from_u64(36),
                DomId::from_u64(44)
            )
        );

        children.remove(DomId::from_u64(35));

        assert_eq!(children.first_child, Some(DomId::from_u64(36)));
        assert_eq!(
            children.get_all(),
            vec!(DomId::from_u64(36), DomId::from_u64(44))
        );

        children.remove(DomId::from_u64(36));

        assert_eq!(children.first_child, Some(DomId::from_u64(44)));
        assert_eq!(children.get_all(), vec!(DomId::from_u64(44)));

        children.remove(DomId::from_u64(44));

        assert_eq!(children.first_child, None);
        assert_eq!(children.get_all(), Vec::<DomId>::new());

        children.insert_before(None, DomId::from_u64(9999));

        assert_eq!(children.first_child, Some(DomId::from_u64(9999)));
        assert_eq!(children.get_all(), vec!(DomId::from_u64(9999)));

        children.insert_before(None, DomId::from_u64(9999));

        assert_eq!(children.first_child, Some(DomId::from_u64(9999)));
        assert_eq!(children.get_all(), vec!(DomId::from_u64(9999)));

        children.insert_before(Some(DomId::from_u64(9999)), DomId::from_u64(8888));

        assert_eq!(children.first_child, Some(DomId::from_u64(8888)));
        assert_eq!(
            children.get_all(),
            vec!(DomId::from_u64(8888), DomId::from_u64(9999))
        );

        children.insert_before(Some(DomId::from_u64(9999)), DomId::from_u64(8888));

        assert_eq!(children.first_child, Some(DomId::from_u64(8888)));
        assert_eq!(
            children.get_all(),
            vec!(DomId::from_u64(8888), DomId::from_u64(9999))
        );

        children.insert_before(Some(DomId::from_u64(9999)), DomId::from_u64(8900));

        assert_eq!(children.first_child, Some(DomId::from_u64(8888)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(8888),
                DomId::from_u64(8900),
                DomId::from_u64(9999)
            )
        );

        children.insert_before(Some(DomId::from_u64(9999)), DomId::from_u64(8900));

        assert_eq!(children.first_child, Some(DomId::from_u64(8888)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(8888),
                DomId::from_u64(8900),
                DomId::from_u64(9999)
            )
        );

        children.insert_before(Some(DomId::from_u64(8900)), DomId::from_u64(9999));

        assert_eq!(children.first_child, Some(DomId::from_u64(8888)));
        assert_eq!(
            children.get_all(),
            vec!(
                DomId::from_u64(8888),
                DomId::from_u64(9999),
                DomId::from_u64(8900)
            )
        );
    }
}
