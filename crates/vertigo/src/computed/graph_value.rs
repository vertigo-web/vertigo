use std::{
    cmp::PartialEq,
    collections::BTreeSet,
    rc::Rc,
};

use crate::{
    computed::{Dependencies, GraphId, GraphRelation}, struct_mut::ValueMut,
};

struct GraphValueDataState<T: PartialEq + 'static> {
    value: Rc<T>,
    _list: GraphRelation,
}

struct GraphValueData<T: PartialEq + 'static> {
    is_computed_type: bool,
    deps: Dependencies,
    id: GraphId,
    get_value_from_parent: Box<dyn Fn() -> (Rc<T>, BTreeSet<GraphId>) + 'static>,
    state: ValueMut<Option<GraphValueDataState<T>>>,
}

impl<T: PartialEq + 'static> GraphValueData<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(
        deps: &Dependencies,
        is_computed_type: bool,
        get_value: F,
    ) -> (GraphId, Rc<GraphValueData<T>>) {
        let id = GraphId::default();

        let get_value = {
            let deps = deps.clone();

            Box::new(move || {
                deps.start_track();
                let result = get_value();
                let parens = deps.stop_track();
                (result, parens)
            })
        };

        let inst = Rc::new(
            GraphValueData {
                is_computed_type,
                deps: deps.clone(),
                id,
                get_value_from_parent: get_value,
                state: ValueMut::new(None),
            }
        );

        (id, inst)
    }

    fn convert_to_relation(&self, edges: BTreeSet<GraphId>) -> GraphRelation {
        GraphRelation::new(self.deps.clone(), edges, self.id)
    }

    fn calculate_new_value(&self) -> (Rc<T>, BTreeSet<GraphId>) {
        let get_value_from_parent = &self.get_value_from_parent;
        get_value_from_parent()
    }

    pub fn get_value(&self, is_computed: bool) -> Rc<T> {
        if is_computed {
            self.deps.report_parent_in_stack(self.id);
        }

        let inner_value = self.state.map(|value| {
            if let Some(value) = value {
                return Some(value.value.clone());
            }
            None
        });

        if let Some(value) = inner_value {
            return value;
        }

        let (new_value, parents_list) = self.calculate_new_value();

        self.state.set(Some(GraphValueDataState {
            value: new_value.clone(),
            _list: self.convert_to_relation(parents_list),
        }));

        new_value
    }

    fn control_refresh(&self) {
        let is_some = self.state.map(|item| item.is_some());

        if is_some {
            let (new_value, parents_list) = self.calculate_new_value();

            self.state.set(Some(GraphValueDataState {
                value: new_value,
                _list: self.convert_to_relation(parents_list),
            }));
        }
    }

    fn control_drop_value(&self) {
        self.state.set(None);
    }
}

trait GraphValueControl {
    fn drop_value(&self);
    fn refresh(&self);
    fn is_computed(&self) -> bool;
}

impl<T: PartialEq + 'static> GraphValueControl for GraphValueData<T> {
    fn drop_value(&self) {
        self.control_drop_value();
    }

    fn refresh(&self) {
        self.control_refresh();
    }

    fn is_computed(&self) -> bool {
        self.is_computed_type
    }
}

#[derive(Clone)]
pub struct GraphValueRefresh { // add type ?
    pub id: GraphId,
    control: Rc<dyn GraphValueControl>,
}

impl GraphValueRefresh {
    fn new(id: GraphId, control: Rc<dyn GraphValueControl>) -> GraphValueRefresh {
        GraphValueRefresh { id, control }
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
}

struct GraphValueInner<T: PartialEq + 'static> {
    id: GraphId,
    inner: Rc<GraphValueData<T>>,
}

impl<T: PartialEq + 'static> GraphValueInner<T> {
    fn new<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, is_computed_type: bool, get_value: F) -> GraphValueInner<T> {

        let (id, graph_value_data) = GraphValueData::new(deps, is_computed_type, get_value);

        let refresh_token = GraphValueRefresh::new(id, graph_value_data.clone());

        deps.refresh_token_add(refresh_token);

        GraphValueInner {
            id,
            inner: graph_value_data,
        }
    }
}

impl<T: PartialEq + 'static> Drop for GraphValueInner<T> {
    fn drop(&mut self) {
        self.inner.deps.refresh_token_drop(self.id);
        self.inner.state.set(None);
        self.inner.deps.external_connections_refresh();
    }
}

pub struct GraphValue<T: PartialEq + 'static> {
    inner: Rc<GraphValueInner<T>>,
}

impl<T: PartialEq + 'static> GraphValue<T> {
    fn new<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, is_computed_type: bool, get_value: F) -> GraphValue<T> {
        GraphValue {
            inner: Rc::new(
                GraphValueInner::new(deps, is_computed_type, get_value)
            )
        }
    }

    pub fn new_computed<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, get_value: F) -> GraphValue<T> {
        GraphValue::new(deps, true, get_value)
    }

    pub fn new_client<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, get_value: F) -> GraphValue<T> {
        GraphValue::new(deps, false, get_value)
    }

    pub fn get_value(&self, is_computed: bool) -> Rc<T> {
        self.inner.inner.get_value(is_computed)
    }

    pub fn deps(&self) -> Dependencies {
        self.inner.inner.deps.clone()
    }

    pub(crate) fn id(&self) -> GraphId {
        self.inner.inner.id
    }
}

impl<T: PartialEq + 'static> Clone for GraphValue<T> {
    fn clone(&self) -> Self {
        GraphValue {
            inner: self.inner.clone(),
        }
    }
}

