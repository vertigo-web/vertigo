use crate::{
    computed::{Computed, GraphId, GraphValue},
    struct_mut::ValueMut, external_connections_refresh,
};

pub struct Client {
    graph_value: GraphValue<()>,
}

impl Client {
    pub fn new<T: Clone, F>(computed: Computed<T>, call: F) -> Client
    where
        T: PartialEq + 'static,
        F: Fn(T) + 'static,
    {
        let graph_value = GraphValue::new(false, {
            let prev_value = ValueMut::new(None);

            move || {
                let value = computed.get();
                let should_update = prev_value.set_and_check(Some(value.clone()));

                if should_update {
                    call(value);
                }
            }
        });

        graph_value.subscribe_value();
        external_connections_refresh();

        Client {
            graph_value,
        }
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}
