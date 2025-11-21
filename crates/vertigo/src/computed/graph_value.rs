use std::rc::Rc;

use crate::{Context, computed::{GraphId, get_dependencies}, struct_mut::ValueMut};

pub struct GraphValue<T> {
    id: GraphId,
    get_value: Box<dyn Fn(&Context) -> T>,
    state: ValueMut<Option<T>>,
}

impl<T: Clone + 'static> GraphValue<T> {
    pub fn new<F: Fn(&Context) -> T + 'static>(
        is_computed_type: bool,
        get_value: F,
    ) -> Rc<GraphValue<T>> {
        let id = match is_computed_type {
            true => GraphId::new_computed(),
            false => GraphId::new_client(),
        };

        let graph_value = Rc::new(GraphValue {
            id,
            get_value: Box::new(get_value),
            state: ValueMut::new(None),
        });

        let weak_value = Rc::downgrade(&graph_value);

        get_dependencies()
            .graph
            .refresh
            .refresh_token_add(graph_value.id, move |kind: bool| {
                if let Some(weak_value) = weak_value.upgrade() {
                    match kind {
                        false => {
                            //false - computed (clear_cache)
                            weak_value.state.set(None);
                        }
                        true => {
                            //true - client (refresh)
                            weak_value.refresh();
                        }
                    }
                }
            });

        graph_value
    }

    fn calculate_new_value(&self) -> T {
        let context = Context::new();
        let new_value = (self.get_value)(&context);
        get_dependencies()
            .graph
            .push_context(self.id, context);

        self.state.set(Some(new_value.clone()));

        new_value
    }

    pub fn get_value(&self, context: &Context) -> T {
        context.add_parent(self.id);

        let inner_value = self.state.map(|value| value.clone());

        if let Some(value) = inner_value {
            return value;
        }

        self.calculate_new_value()
    }

    fn refresh(&self) {
        let is_some = self.state.map(|item| item.is_some());

        if is_some {
            self.calculate_new_value();
        }
    }

    pub(crate) fn id(&self) -> GraphId {
        self.id
    }
}

impl<T> Drop for GraphValue<T> {
    fn drop(&mut self) {
        let deps = get_dependencies();
        deps.graph.refresh.refresh_token_drop(self.id);
        deps.graph.remove_client(self.id);
    }
}
