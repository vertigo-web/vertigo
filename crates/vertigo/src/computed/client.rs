use crate::{
    computed::{Computed, GraphId, GraphValue},
    struct_mut::ValueMut, get_driver,
};

use super::context::Context;

pub struct Client {
    graph_value: GraphValue<()>,
}

impl Client {
    pub fn new<T, R, F>(computed: Computed<T>, call: F) -> Client
    where
        R: 'static,
        T: Clone + PartialEq + 'static,
        F: Fn(T) -> R + 'static,
    {   
        let prev_value = ValueMut::new(None);
        let deps = get_driver().inner.dependencies.clone();

        let context = Context::new();
        let resource_box = ValueMut::new(None);

        let graph_value = GraphValue::new(false, move || {
            let value = computed.get(&context);
            let should_update = prev_value.set_and_check(Some(value.clone()));

            if should_update {
                deps.block_tracking_on();
                let resource = call(value);
                resource_box.change(move |inner| {
                    *inner = Some(resource);
                });
                deps.block_tracking_off();
            }
        });

        graph_value.get_value(false);

        // graph_value.subscribe_value();
        get_driver().inner.dependencies.external_connections_refresh();

        Client {
            graph_value,
        }
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}
