use crate::computed::{
    Dependencies,
    GraphId,
};

pub struct GraphRelation {
    deps: Dependencies,
    parent_id: GraphId,
    client_id: GraphId,
}

impl GraphRelation {
    pub fn new(deps: Dependencies, parent_id: GraphId, client_id: GraphId) -> GraphRelation {

        deps.add_graph_connection(parent_id, client_id);

        GraphRelation {
            deps,
            parent_id,
            client_id,
        }
    }
}

impl Drop for GraphRelation {
    fn drop(&mut self) {
        self.deps.remove_graph_connection(self.parent_id, self.client_id);
    }
}

