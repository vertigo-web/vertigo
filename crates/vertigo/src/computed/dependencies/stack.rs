use std::collections::{BTreeSet};

use crate::{computed::graph_id::GraphId, struct_mut::VecDequeMut};

pub struct Stack {
    stack_relations: VecDequeMut<BTreeSet<GraphId>>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack_relations: VecDequeMut::new(),
        }
    }

    pub fn start_track(&self) {
        let stack_frame = BTreeSet::new();
        self.stack_relations.push_back(stack_frame);
    }

    pub fn report_parent_in_stack(&self, parent_id: GraphId) {
        let len = self.stack_relations.len();

        if len < 1 {
            //log::warn!("frame with stack - not found len=0");
            return;
        }

        let last_index = len - 1;

        self.stack_relations.get_mut(last_index, |last_item| {
            match last_item {
                Some(last_item) => {
                    last_item.insert(parent_id);
                }
                None => {
                    log::warn!("frame with stack - not found get_mut=None");
                }
            }
        });
    }

    pub fn stop_track(&self) -> BTreeSet<GraphId> {
        let last_item = self.stack_relations.pop_back();

        if let Some(last_item) = last_item {
            return last_item;
        };

        log::error!("the stack frame was expected");
        BTreeSet::new()
    }
}
