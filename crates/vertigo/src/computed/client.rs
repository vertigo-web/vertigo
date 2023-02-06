use crate::{
    computed::{Computed, GraphId, GraphValue},
    struct_mut::ValueMut,
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

        let resource_box = ValueMut::new(None);

        let graph_value = GraphValue::new(false, move |context| {
            let value = computed.get(context);

            let should_update = prev_value.set_and_check(Some(value.clone()));

            if should_update {
                let resource = call(value);
                resource_box.change(move |inner| {
                    *inner = Some(resource);
                });
            }
        });

        let context = Context::new();
        graph_value.get_value(&context);
        let _ = context;

        Client {
            graph_value,
        }
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}
