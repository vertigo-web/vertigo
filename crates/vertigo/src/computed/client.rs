use std::rc::Rc;

use crate::computed::{
    Dependencies,
    Computed,
    GraphValue,
    GraphId,
};
use crate::utils::{EqBox, BoxRefCell};

#[derive(PartialEq)]
pub struct Client {
    graph_value: EqBox<GraphValue<()>>,
}

impl Client {
    pub fn new<T: PartialEq + 'static, F: Fn(&T) + 'static>(deps: Dependencies, computed: Computed<T>, call: F) -> Client {
        let graph_value = GraphValue::new_client(&deps, {
            let prev_value: BoxRefCell<Option<Rc<T>>> = BoxRefCell::new(None, "client - prev state");

            move || {
                let value = computed.get_value();

                let should_update = prev_value.change(value.clone(), |state, value| {
                    if *state == Some(value.clone()) {
                        false
                    } else {
                        *state = Some(value);
                        true
                    }
                });

                if should_update {
                    call(value.as_ref());
                }

                Rc::new(())
            }
        });

        let _ = graph_value.get_value(false);

        Client {
            graph_value: EqBox::new(graph_value)
        }
    }

    pub fn id(&self) -> GraphId {
        self.graph_value.id()
    }
}
