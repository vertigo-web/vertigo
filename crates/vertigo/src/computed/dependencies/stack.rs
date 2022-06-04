use std::collections::BTreeSet;

use crate::{computed::graph_id::GraphId, struct_mut::{VecDequeMut, VecMut}};

#[derive(PartialEq, Eq, Debug)]
enum StackKind {
    Normal,
    BuildingDom,
    IgnoreTracking,
}

trait StackFrame {
    fn insert(&mut self, parent_id: GraphId);
    fn get_edges(&mut self) -> (BTreeSet<GraphId>, GraphId);
    fn get_kind(&self) -> StackKind;
}

struct StackFrameNormal {
    edges: BTreeSet<GraphId>,
    client_id: GraphId,
}

impl StackFrameNormal {
    pub fn trace(client_id: GraphId) -> StackFrameNormal {
        StackFrameNormal {
            edges: BTreeSet::new(),
            client_id,
        }
    }
}

impl StackFrame for StackFrameNormal {
    fn insert(&mut self, parent_id: GraphId) {
        self.edges.insert(parent_id);
    }

    fn get_edges(&mut self) -> (BTreeSet<GraphId>, GraphId) {
        let edges = std::mem::take(&mut self.edges);
        let client_id = self.client_id;

        (edges, client_id)
    }

    fn get_kind(&self) -> StackKind {
        StackKind::Normal
    }
}

struct StackFrameBuildingDom {
}

impl StackFrameBuildingDom {
    pub fn new() -> StackFrameBuildingDom {
        StackFrameBuildingDom {}
    }
}

impl StackFrame for StackFrameBuildingDom {
    fn insert(&mut self, _parent_id: GraphId) {
        log::error!("During the dob building process, reactive values cannot be read");
    }

    fn get_edges(&mut self) -> (BTreeSet<GraphId>, GraphId) {
        panic!("edges could not be read because tracing was blocked");
    }

    fn get_kind(&self) -> StackKind {
        StackKind::BuildingDom
    }
}

struct StackFrameIgnoreTracking {
}

impl StackFrameIgnoreTracking {
    pub fn new() -> StackFrameIgnoreTracking {
        StackFrameIgnoreTracking {}
    }
}

impl StackFrame for StackFrameIgnoreTracking {
    fn insert(&mut self, _parent_id: GraphId) {
    }

    fn get_edges(&mut self) -> (BTreeSet<GraphId>, GraphId) {
        panic!("edges could not be read because tracing was blocked");
    }

    fn get_kind(&self) -> StackKind {
        StackKind::IgnoreTracking
    }
}

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
    closed_relations: VecMut<StackCommand>,
    stack_relations: VecDequeMut<Box<dyn StackFrame>>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            closed_relations: VecMut::new(),
            stack_relations: VecDequeMut::new(),
        }
    }

    pub fn start_track(&self, client_id: GraphId) {
        self.stack_relations.push_back(Box::new(StackFrameNormal::trace(client_id)));
    }

    pub fn report_parent_in_stack(&self, parent_id: GraphId) {
        let len = self.stack_relations.len();

        if len < 1 {
            log::warn!("frame with stack - not found len=0 - The value was read outside the block tracking subscriptions");
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

    pub fn stop_track(&self) {
        let last_item = self.stack_relations.pop_back();

        if let Some(mut last_item) = last_item {
            let (parents, client) = last_item.get_edges();
            self.closed_relations.push(StackCommand::Set { parent_ids: parents, client_id: client });
            return;
        };

        log::error!("the stack frame was expected");
    }

    pub fn block_tracking_on(&self) {
        self.stack_relations.push_back(Box::new(StackFrameBuildingDom::new()));
    }

    pub fn block_tracking_off(&self) {
        let last_item = self.stack_relations.pop_back();

        if let Some(last_item) = last_item {
            assert_eq!(last_item.get_kind(), StackKind::BuildingDom);
            return;
        }

        log::error!("the stack frame was expected");
    }

    pub fn ignore_tracking_on(&self) {
        self.stack_relations.push_back(Box::new(StackFrameIgnoreTracking::new()));
    }

    pub fn ignore_tracking_off(&self) {
        let last_item = self.stack_relations.pop_back();

        if let Some(last_item) = last_item {
            assert_eq!(last_item.get_kind(), StackKind::IgnoreTracking);
            return;
        }

        log::error!("the stack frame was expected");
    }

    pub fn get_command(&self) -> Vec<StackCommand> {
        self.closed_relations.take()   
    }

    pub fn remove_client(&self, client_id: GraphId) {
        self.closed_relations.push(StackCommand::Remove { client_id });
    }

    pub fn is_empty(&self) -> bool {
        self.stack_relations.is_empty()
    }
}
