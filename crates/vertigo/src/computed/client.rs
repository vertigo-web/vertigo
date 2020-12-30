use std::rc::Rc;
use crate::computed::{
    Dependencies,
    Computed,
    GraphValue,
    GraphId,
};
use std::cmp::PartialEq;

pub struct Client {
    graph_value: GraphValue<()>,
}

impl Client {
    pub fn new<T: PartialEq + 'static, F: Fn(&T) + 'static>(deps: Dependencies, computed: Computed<T>, call: F) -> Client {
        let graph_value = GraphValue::new_client(&deps, move || {
            let value = computed.get_value();
            call(value.as_ref());
            Rc::new(())
        });

        let _ = graph_value.get_value();

        Client {
            graph_value
        }
    }

    pub fn off(self: Client) {
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}

