use crate::{
    computed::{Computed, GraphId, GraphValue},
    struct_mut::ValueMut, external_connections_refresh,
};

use super::context::Context;

pub struct Client {
    graph_value: GraphValue<()>,
}

impl Client {
    pub fn new<T, F>(computed: Computed<T>, call: F) -> Client
    where
        T: Clone + PartialEq + 'static,
        F: Fn(T) + 'static,
    {   
        let prev_value = ValueMut::new(None);
        let context = Context::new();

        let graph_value = GraphValue::new(false, move || {
            let value = computed.get(&context);
            let should_update = prev_value.set_and_check(Some(value.clone()));

            if should_update {
                call(value);
            }
        });

        graph_value.get_value(false);

        // graph_value.subscribe_value();
        external_connections_refresh();

        Client {
            graph_value,
        }
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}
