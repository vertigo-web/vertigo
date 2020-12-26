use std::collections::{VecDeque, BTreeSet};
use crate::computed::graph_id::GraphId;

pub struct Stack {
    stack_relations: VecDeque<BTreeSet<GraphId>>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack_relations: VecDeque::new(),
        }
    }

    pub fn start_track(&mut self) {
        let stack_frame = BTreeSet::new();
        self.stack_relations.push_back(stack_frame);
    }

    pub fn report_parent_in_stack(&mut self, parent_id: GraphId) {
        let len = self.stack_relations.len();

        if len < 1 {
            log::warn!("frame with stack - not found len=0");
            return;
        }

        let last_index = len - 1;
        let last_item = self.stack_relations.get_mut(last_index);

        match last_item {
            Some(last_item) => {
                last_item.insert(parent_id);
            },
            None => {
                log::warn!("frame with stack - not found get_mut=None");
            }
        }
    }

    pub fn stop_track(&mut self) -> BTreeSet<GraphId> {
        let last_item = self.stack_relations.pop_back();

        if let Some(last_item) = last_item {
            return last_item;
        };

        log::error!("the stack frame was expected");
        BTreeSet::new()
    }
}
