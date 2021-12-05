use std::rc::Rc;
use std::collections::{BTreeSet};
use std::cmp::PartialEq;

use crate::computed::{
    Dependencies,
    GraphId,
    GraphRelation,
};
use crate::utils::BoxRefCell;

#[derive(PartialEq)]
enum GraphValueType {
    Computed,
    Client,
}

impl GraphValueType {
    fn is_computed(&self) -> bool {
        match self {
            Self::Computed => true,
            Self::Client => false,
        }
    }
}

struct GraphValueDataState<T: PartialEq + 'static> {
    value: Rc<T>,
    _list: Vec<GraphRelation>,
}

struct GraphValueData<T: PartialEq + 'static> {
    is_drop: bool,
    value_type: GraphValueType,
    deps: Dependencies,
    id: GraphId,
    get_value_from_parent: Box<dyn Fn() -> (Rc<T>, BTreeSet<GraphId>) + 'static>,
    state: Option<GraphValueDataState<T>>,
}

impl<T: PartialEq + 'static> GraphValueData<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, value_type: GraphValueType, get_value: F) -> (GraphId, Rc<BoxRefCell<GraphValueData<T>>>) {

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
            BoxRefCell::new(
                GraphValueData {
                    is_drop: false,
                    value_type,
                    deps: deps.clone(),
                    id,
                    get_value_from_parent: get_value,
                    state: None,
                },
                "GraphValueData",
            )
        );

        (id, inst)
    }

    fn convert_to_relation(&self, edges: BTreeSet<GraphId>) -> Vec<GraphRelation> {
        let mut list_relations: Vec<GraphRelation> = Vec::new();

        for parent_id in edges {
            list_relations.push(GraphRelation::new(self.deps.clone(), parent_id, self.id));
        }

        list_relations
    }

    fn calculate_new_value(&self) -> (Rc<T>, BTreeSet<GraphId>) {
        let get_value_from_parent = &self.get_value_from_parent;
        get_value_from_parent()
    }

    pub fn get_value(&mut self, is_computed: bool) -> Rc<T> {
        if is_computed {
            self.deps.report_parent_in_stack(self.id);
        }

        if let Some(state) = &self.state {
            return state.value.clone();
        }

        let (new_value, parents_list) = self.calculate_new_value();

        self.state = Some(GraphValueDataState {
            value: new_value.clone(),
            _list: self.convert_to_relation(parents_list)
        });

        new_value
    }

    pub fn refresh(&mut self) {
        if self.value_type.is_computed() {
            log::error!("The refresh function should never be executed on a client node");
            return;
        }

        if self.is_drop {
            log::info!("Client unsubscribed, skip refreshing");
            return;
        }

        let (new_value, parents_list) = self.calculate_new_value();

        self.state = Some(GraphValueDataState {
            value: new_value,
            _list: self.convert_to_relation(parents_list)
        });
    }

    pub fn drop_value(&mut self) {
        if self.state.is_none() {
            log::error!("Incoherent state");
            return;
        }

        self.state = None;
    }

    fn is_computed(&self) -> bool {
        self.value_type == GraphValueType::Computed
    }
}

pub trait GraphValueControl {
    fn drop_value(&self);
    fn refresh(&self);
    fn is_computed(&self) -> bool;
}

impl<T: PartialEq + 'static> GraphValueControl for BoxRefCell<GraphValueData<T>> {
    fn drop_value(&self) {
        self.change((), |state, _| {
            state.drop_value();
        });
    }

    fn refresh(&self) {
        self.change((), |state, _| {
            state.refresh()
        })
    }

    fn is_computed(&self) -> bool {
        self.change((), |state, _| {
            state.is_computed()
        })
    }
}


#[derive(Clone)]
pub struct GraphValueRefresh {              //add type ?
    pub id: GraphId,
    control: Rc<dyn GraphValueControl>,
}

impl GraphValueRefresh {
    pub fn new(id: GraphId, control: Rc<dyn GraphValueControl>) -> GraphValueRefresh {
        GraphValueRefresh {
            id,
            control,
        }
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
    inner: Rc<BoxRefCell<GraphValueData<T>>>,
}

impl<T: PartialEq + 'static> GraphValueInner<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, value_type: GraphValueType, get_value: F) -> Rc<GraphValueInner<T>> {

        let (id, graph_value_data) = GraphValueData::new(deps, value_type, get_value);

        let refresh_token = GraphValueRefresh::new(id, graph_value_data.clone());

        deps.refresh_token_add(refresh_token);

        Rc::new(
            GraphValueInner {
                id,
                inner: graph_value_data
            }
        )
    }
}

impl<T: PartialEq + 'static> Drop for GraphValueInner<T> {
    fn drop(&mut self) {
        let deps = self.inner.change((), |state, _| {
            state.is_drop = true;
            state.deps.clone()
        });

        deps.refresh_token_drop(self.id);
    }
}


pub struct GraphValue<T: PartialEq + 'static> {
    inner: Rc<GraphValueInner<T>>,
}

impl<T: PartialEq + 'static> Clone for GraphValue<T> {
    fn clone(&self) -> Self {
        GraphValue {
            inner: self.inner.clone(),
        }
    }
}

impl<T: PartialEq + 'static> GraphValue<T> {
    fn new<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, value_type: GraphValueType, get_value: F) -> GraphValue<T> {
        GraphValue {
            inner: GraphValueInner::new(deps, value_type, get_value)
        }
    }

    pub fn new_computed<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, get_value: F) -> GraphValue<T> {
        GraphValue::new(deps, GraphValueType::Computed, get_value)
    }

    pub fn new_client<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, get_value: F) -> GraphValue<T> {
        GraphValue::new(deps, GraphValueType::Client, get_value)
    }

    pub fn is_computed(&self) -> bool {
        self.inner.inner.get(|state| {
            state.value_type == GraphValueType::Computed
        })
    }

    pub fn is_client(&self) -> bool {
        self.inner.inner.get(|state| {
            state.value_type == GraphValueType::Client
        })
    }

    pub fn get_value(&self, is_computed: bool) -> Rc<T> {
        self.inner.inner.change(is_computed, |state, is_computed| {
            state.get_value(is_computed)
        })
    }

    pub fn deps(&self) -> Dependencies {
        self.inner.inner.get(|state| {
            state.deps.clone()
        })
    }

    pub(crate) fn id(&self) -> GraphId {
        self.inner.inner.get(|state| {
            state.id
        })
    }
}

