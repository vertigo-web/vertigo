use std::collections::HashMap;

struct ChildrenNode {
    left: u64,
    right: u64,
}

pub struct ElementChildren {
    first_child: Option<u64>,
    childs: HashMap<u64, ChildrenNode>,
}

impl ElementChildren {
    pub fn new() -> ElementChildren {
        ElementChildren {
            first_child: None,
            childs: HashMap::new(),
        }
    }

    pub fn childs(&self) -> Vec<u64> {
        if self.childs.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();

        let Some(mut wsk_id) = self.first_child else {
            unreachable!();
        };

        for _ in 0..self.childs.len() {
            let Some(node) = self.childs.get(&wsk_id) else {
                unreachable!();
            };

            result.push(wsk_id);
            wsk_id = node.right;
        }

        //We expect to turn a full circle
        assert_eq!(Some(wsk_id), self.first_child);

        result
    }

    fn insert_on_left(&mut self, node_right_id: u64, node_id: u64) {
        let Some(node_right) = self.childs.get_mut(&node_right_id) else {
            unreachable!();
        };

        let node_left_id = node_right.left;
        node_right.left = node_id;

        let Some(node_left) = self.childs.get_mut(&node_left_id) else {
            unreachable!();
        };
        node_left.right = node_id;

        self.childs.insert(node_id, ChildrenNode {
            left: node_left_id,
            right: node_right_id,
        });
    }

    pub fn insert_before(&mut self, ref_id: Option<u64>, child: u64) {
        if ref_id == Some(child) {
            panic!("ref_id must not be equal to child_id, ref_id={ref_id:?}, child={child:?}");
        }

        self.remove(child);

        let right_node_id = {
            if let Some(ref_id) = ref_id {
                ref_id
            } else if let Some(first_child) = self.first_child {
                first_child
            } else {
                assert_eq!(self.childs.len(), 0);

                self.first_child = Some(child);
                self.childs.insert(child, ChildrenNode {
                    left: child,
                    right: child,
                });
                return;
            }
        };

        self.insert_on_left(right_node_id, child);

        if self.first_child == ref_id {
            self.first_child = Some(child);
        }
    }

    pub fn remove(&mut self, node_id: u64) {
        let Some(node) = self.childs.remove(&node_id) else {
            return;
        };

        if node.left == node_id {
            assert_eq!(node.right, node_id);

            self.first_child = None;
            return;
        }

        let Some(node_right) = self.childs.get_mut(&node.right) else {
            unreachable!();
        };

        node_right.left = node.left;

        let Some(node_left) = self.childs.get_mut(&node.left) else {
            unreachable!();
        };

        node_left.right = node.right;

        if self.first_child == Some(node_id) {
            self.first_child = Some(node.right);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::ElementChildren;

    #[test]
    fn test_children() {
        let mut children = ElementChildren::new();

        assert_eq!(children.first_child, None);
        assert_eq!(children.childs(), Vec::<u64>::new());

        children.insert_before(None, 44);

        assert_eq!(children.first_child, Some(44));
        assert_eq!(children.childs(), vec!(44));

        children.insert_before(None, 55);

        assert_eq!(children.first_child, Some(44));
        assert_eq!(children.childs(), vec!(44, 55));

        children.insert_before(None, 66);

        assert_eq!(children.first_child, Some(44));
        assert_eq!(children.childs(), vec!(44, 55, 66));

        children.insert_before(Some(44), 33);

        assert_eq!(children.first_child, Some(33));
        assert_eq!(children.childs(), vec!(33, 44, 55, 66));

        children.insert_before(Some(44), 35);

        assert_eq!(children.first_child, Some(33));
        assert_eq!(children.childs(), vec!(33, 35, 44, 55, 66));

        children.remove(55);

        assert_eq!(children.first_child, Some(33));
        assert_eq!(children.childs(), vec!(33, 35, 44, 66));

        children.remove(66);

        assert_eq!(children.first_child, Some(33));
        assert_eq!(children.childs(), vec!(33, 35, 44));

        children.remove(33);

        assert_eq!(children.first_child, Some(35));
        assert_eq!(children.childs(), vec!(35, 44));

        children.insert_before(Some(44), 36);

        assert_eq!(children.first_child, Some(35));
        assert_eq!(children.childs(), vec!(35, 36, 44));

        children.remove(35);

        assert_eq!(children.first_child, Some(36));
        assert_eq!(children.childs(), vec!(36, 44));

        children.remove(36);

        assert_eq!(children.first_child, Some(44));
        assert_eq!(children.childs(), vec!(44));

        children.remove(44);

        assert_eq!(children.first_child, None);
        assert_eq!(children.childs(), Vec::<u64>::new());

        children.insert_before(None, 9999);

        assert_eq!(children.first_child, Some(9999));
        assert_eq!(children.childs(), vec!(9999));

        children.insert_before(None, 9999);

        assert_eq!(children.first_child, Some(9999));
        assert_eq!(children.childs(), vec!(9999));

        children.insert_before(Some(9999), 8888);

        assert_eq!(children.first_child, Some(8888));
        assert_eq!(children.childs(), vec!(8888, 9999));

        children.insert_before(Some(9999), 8888);

        assert_eq!(children.first_child, Some(8888));
        assert_eq!(children.childs(), vec!(8888, 9999));

        children.insert_before(Some(9999), 8900);

        assert_eq!(children.first_child, Some(8888));
        assert_eq!(children.childs(), vec!(8888, 8900, 9999));

        children.insert_before(Some(9999), 8900);

        assert_eq!(children.first_child, Some(8888));
        assert_eq!(children.childs(), vec!(8888, 8900, 9999));

        children.insert_before(Some(8900), 9999);

        assert_eq!(children.first_child, Some(8888));
        assert_eq!(children.childs(), vec!(8888, 9999, 8900));
    }
}
