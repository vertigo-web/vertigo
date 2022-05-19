use std::{
    rc::Rc,
};

use crate::{
    computed::{Dependencies, GraphId}, struct_mut::ValueMut, get_dependencies,
};


struct GraphValueData<T> {
    is_computed_type: bool,
    deps: Dependencies,
    id: GraphId,
    get_value: Box<dyn Fn() -> T>,
    state: ValueMut<Option<T>>,
}

impl<T: Clone> GraphValueData<T> {
    pub fn new<F: Fn() -> T + 'static>(
        deps: &Dependencies,
        is_computed_type: bool,
        get_value: F,
    ) -> Rc<GraphValueData<T>> {
        let id = GraphId::default();

        Rc::new(
            GraphValueData {
                is_computed_type,
                deps: deps.clone(),
                id,
                get_value: Box::new(get_value),
                state: ValueMut::new(None),
            }
        )
    }

    fn calculate_new_value(&self) -> T {
        self.deps.start_track();
        let get_value = &self.get_value;
        let new_value = get_value();
        let parents_list = self.deps.stop_track();

        self.state.set(Some(new_value.clone()));
        self.deps.set_parent_for_client(self.id, parents_list);

        new_value
    }

    pub fn get_value(&self) -> T {
        self.deps.report_parent_in_stack(self.id);

        let inner_value = self.state.map(|value| value.clone());

        if let Some(value) = inner_value {
            return value;
        }

        self.calculate_new_value()
    }

    pub fn subscribe_value(&self) {
        self.calculate_new_value();
    }

    fn control_refresh(&self) {
        let is_some = self.state.map(|item| item.is_some());

        if is_some {
            self.calculate_new_value();
        }
    }

    fn control_drop_value(&self) {
        self.state.set(None);
        self.deps.remove_client(self.id);
    }
}

trait GraphValueControl {
    fn drop_value(&self);
    fn refresh(&self);
    fn is_computed(&self) -> bool;
    fn id(&self) -> GraphId;
}

impl<T: Clone> GraphValueControl for GraphValueData<T> {
    fn drop_value(&self) {
        self.control_drop_value();
    }

    fn refresh(&self) {
        self.control_refresh();
    }

    fn is_computed(&self) -> bool {
        self.is_computed_type
    }

    fn id(&self) -> GraphId {
        self.id
    }
}

#[derive(Clone)]
pub struct GraphValueRefresh { // add type ?
    control: Rc<dyn GraphValueControl>,
}

impl GraphValueRefresh {
    fn new(control: Rc<dyn GraphValueControl>) -> GraphValueRefresh {
        GraphValueRefresh { control }
    }

    pub fn drop_value(&self) {
        self.control.drop_value();
    }

    pub fn refresh(&self) {
        self.control.refresh()
    }

    pub fn is_computed(&self) -> bool {
        self.control.is_computed()
    }

    pub fn id(&self) -> GraphId {
        self.control.id()
    }
}

struct GraphValueInner<T: Clone> {
    inner: Rc<GraphValueData<T>>,
}

impl<T: Clone + 'static> GraphValueInner<T> {
    fn new<F: Fn() -> T + 'static>(is_computed_type: bool, get_value: F) -> GraphValueInner<T> {
        let deps = get_dependencies();

        let graph_value = GraphValueData::new(&deps, is_computed_type, get_value);

        deps.refresh_token_add(GraphValueRefresh::new(graph_value.clone()));

        GraphValueInner {
            inner: graph_value,
        }
    }
}

impl<T: Clone> Drop for GraphValueInner<T> {
    fn drop(&mut self) {
        self.inner.deps.refresh_token_drop(self.inner.id);
        self.inner.control_drop_value();
        self.inner.deps.external_connections_refresh();
    }
}

pub struct GraphValue<T: Clone> {
    inner: Rc<GraphValueInner<T>>,
}

impl<T: Clone + 'static> GraphValue<T> {
    pub fn new<F: Fn() -> T + 'static>(is_computed_type: bool, get_value: F) -> GraphValue<T> {
        GraphValue {
            inner: Rc::new(
                GraphValueInner::new(is_computed_type, get_value)
            )
        }
    }

    pub fn get_value(&self) -> T {
        self.inner.inner.get_value()
    }

    pub fn subscribe_value(&self) {
        self.inner.inner.subscribe_value();
    }

    pub(crate) fn id(&self) -> GraphId {
        self.inner.inner.id
    }
}

impl<T: Clone> Clone for GraphValue<T> {
    fn clone(&self) -> Self {
        GraphValue {
            inner: self.inner.clone(),
        }
    }
}

