use std::{
    rc::Rc,
};

use crate::{
    computed::{
        Dependencies, GraphId
    },
    struct_mut::ValueMut,
    get_driver,
};

struct GraphValueData<T> {
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
        let id = match is_computed_type {
            true => GraphId::new_computed(),
            false => GraphId::new_client(),
        };

        Rc::new(
            GraphValueData {
                deps: deps.clone(),
                id,
                get_value: Box::new(get_value),
                state: ValueMut::new(None),
            }
        )
    }

    fn calculate_new_value(&self) -> T {
        self.deps.start_track(self.id);
        let new_value = (self.get_value)();
        self.deps.stop_track();

        self.state.set(Some(new_value.clone()));

        new_value
    }

    pub fn get_value(&self, report_parent: bool) -> T {
        if report_parent {
            self.deps.report_parent_in_stack(self.id);
        }

        let inner_value = self.state.map(|value| value.clone());

        if let Some(value) = inner_value {
            return value;
        }

        self.calculate_new_value()
    }
}

trait GraphValueControl {
    fn id(&self) -> GraphId;
    fn clear_cache(&self);          //for Computed
    fn refresh(&self);              //for Client
}

impl<T: Clone> GraphValueControl for GraphValueData<T> {
    fn id(&self) -> GraphId {
        self.id
    }

    fn clear_cache(&self) {
        self.state.set(None);
    }

    fn refresh(&self) {
        let is_some = self.state.map(|item| item.is_some());

        if is_some {
            self.calculate_new_value();
        }
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

    pub fn id(&self) -> GraphId {
        self.control.id()
    }

    pub fn clear_cache(&self) {
        self.control.clear_cache();
    }

    pub fn refresh(&self) {
        self.control.refresh()
    }
}

struct GraphValueInner<T: Clone> {
    inner: Rc<GraphValueData<T>>,
}

impl<T: Clone + 'static> GraphValueInner<T> {
    fn new<F: Fn() -> T + 'static>(is_computed_type: bool, get_value: F) -> GraphValueInner<T> {
        let deps = get_driver().inner.dependencies.clone();

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
        self.inner.deps.remove_client(self.inner.id);
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

    pub fn get_value(&self, report_parent: bool) -> T {
        self.inner.inner.get_value(report_parent)
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

