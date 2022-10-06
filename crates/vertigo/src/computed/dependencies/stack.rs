use std::collections::BTreeSet;

use crate::{computed::graph_id::GraphId, struct_mut::VecMut, Context};

#[derive(Debug)]
pub enum StackCommand {
    Set {
        parent_ids: BTreeSet<GraphId>,
        client_id: GraphId,
    },
    Remove {
        client_id: GraphId,
    }
}

pub struct Stack {
    command: VecMut<StackCommand>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            command: VecMut::new(),
        }
    }

    pub fn push_track(&self, client_id: GraphId, context: Context) {
        let parents = context.get_parents();
        self.command.push(StackCommand::Set { parent_ids: parents, client_id });
    }

    pub fn remove_client(&self, client_id: GraphId) {
        self.command.push(StackCommand::Remove { client_id });
    }

    pub fn get_command(&self) -> Vec<StackCommand> {
        self.command.take()   
    }
}
