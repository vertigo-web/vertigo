use std::rc::Rc;

use crate::{
    computed::{Computed, Dependencies, GraphId, GraphValue},
    utils::{EqBox}, struct_mut::ValueMut,
};

#[derive(PartialEq)]
pub struct Client {
    graph_value: EqBox<GraphValue<()>>,
}

impl Client {
    pub fn new<T, F>(deps: Dependencies, computed: Computed<T>, call: F) -> Client
    where
        T: PartialEq + 'static,
        F: Fn(&T) + 'static,
    {
        let graph_value = GraphValue::new_client(&deps, {
            let prev_value = ValueMut::new(None);

            move || {
                let value = computed.get_value();
                let should_update = prev_value.set_and_check(Some(value.clone()));

                if should_update {
                    call(value.as_ref());
                }

                Rc::new(())
            }
        });

        let _ = graph_value.get_value(false);
        deps.external_connections_refresh();

        Client {
            graph_value: EqBox::new(graph_value),
        }
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}
