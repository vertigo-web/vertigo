use std::rc::Rc;
use std::collections::BTreeSet;
use std::cmp::PartialEq;
use crate::computed::{
    BoxRefCell,
    Dependencies,
    GraphId,
    GraphRelation,
};

#[derive(PartialEq)]
enum GraphValueType {
    Computed,
    Client,
}

#[derive(Clone)]
pub struct GraphValueRefresh {
    pub id: GraphId,
    //type ?
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

    pub fn refresh(&self) -> bool {
        self.control.refresh()
    }

    pub fn is_computed(&self) -> bool {
        self.control.is_computed()
    }
}

pub trait GraphValueControl {
    fn drop_value(&self);
    fn refresh(&self) -> bool;               //true - value is new
    fn is_computed(&self) -> bool;
}

struct GraphValueState<T: PartialEq> {
    value: Rc<T>,
    _list: Vec<GraphRelation>,
}

struct GraphValueInner<T: PartialEq> {
    value_type: GraphValueType,
    deps: Dependencies,
    id: GraphId,
    get_value_from_parent: Box<dyn Fn() -> (Rc<T>, BTreeSet<GraphId>) + 'static>,
    state: Option<GraphValueState<T>>,
}

impl<T: PartialEq> GraphValueInner<T> {
    fn convert_to_relation(&self, edges: BTreeSet<GraphId>) -> Vec<GraphRelation> {
        let mut list_relations: Vec<GraphRelation> = Vec::new();

        for parent_id in edges {
            list_relations.push(GraphRelation::new(self.deps.clone(), parent_id, self.id.clone()));
        }

        list_relations
    }

    fn calculate_new_value(&self) -> (Rc<T>, BTreeSet<GraphId>) {
        let get_value_from_parent = &self.get_value_from_parent;
        get_value_from_parent()
    }

    pub fn get_value(&mut self) -> (Rc<T>, bool) {
        self.deps.report_parent_in_stack(self.id);

        if let Some(state) = &self.state {
            return (state.value.clone(), false);
        }

        let (new_value, parents_list) = self.calculate_new_value();

        self.state = Some(GraphValueState {
            value: new_value.clone(),
            _list: self.convert_to_relation(parents_list)
        });

        (new_value, true)
    }

    pub fn refresh(&mut self) -> bool {
        if let Some(state) = &self.state {
            let (new_value, parents_list) = self.calculate_new_value();

            if new_value != state.value {
                self.state = Some(GraphValueState {
                    value: new_value,
                    _list: self.convert_to_relation(parents_list)
                });
                return true;
            }

            return false;

        } else {
            log::error!("Incoherent state");
        }

        false
    }

    pub fn drop_value(&mut self) {
        if self.state.is_none() {
            log::error!("Incoherent state");
            return;
        }

        self.state = None;
    }
}

pub struct GraphValue<T: PartialEq + 'static> {
    inner: Rc<BoxRefCell<GraphValueInner<T>>>,
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

        GraphValue {
            inner: Rc::new(
                BoxRefCell::new(
                    GraphValueInner {
                        value_type,
                        deps: deps.clone(),
                        id,
                        get_value_from_parent: get_value,
                        state: None,
                    }
                )
            )
        }
    }

    pub fn new_computed<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, get_value: F) -> GraphValue<T> {
        GraphValue::new(deps, GraphValueType::Computed, get_value)
    }

    pub fn new_client<F: Fn() -> Rc<T> + 'static>(deps: &Dependencies, get_value: F) -> GraphValue<T> {
        GraphValue::new(deps, GraphValueType::Client, get_value)
    }

    pub fn is_computed(&self) -> bool {
        self.inner.get(|state| {
            state.value_type == GraphValueType::Computed
        })
    }

    pub fn is_client(&self) -> bool {
        self.inner.get(|state| {
            state.value_type == GraphValueType::Client
        })
    }

    pub fn get_value(&self) -> Rc<T> {
        let (value, is_fresh) = self.inner.change_no_params(|state| {
            state.get_value()
        });

        if is_fresh {
            let id = self.inner.get(|state| {
                state.id.clone()
            });

            let self_clone = GraphValueRefresh::new(id, Rc::new((*self).clone()));

            self.inner.change(self_clone, |state, self_clone| {
                state.deps.report_graph_value_as_refresh_token(self_clone);
            })
        }

        value
    }

    pub fn deps(&self) -> Dependencies {
        self.inner.get(|state| {
            state.deps.clone()
        })
    }

    pub fn refresh(&self) -> bool {
        self.inner.change_no_params(|state| {
            state.refresh()
        })
    }

    pub fn drop_value_inner(&self) {
        self.inner.change_no_params(|state| {
            state.drop_value();
        })
    }
    
    pub fn id(&self) -> GraphId {
        self.inner.get(|state| {
            state.id.clone()
        })
    }
}

impl<T: PartialEq> GraphValueControl for GraphValue<T> {
    fn drop_value(&self) {
        self.drop_value_inner();
    }

    fn refresh(&self) -> bool {
        self.refresh() 
    }

    fn is_computed(&self) -> bool {
        self.is_computed()
    }
}
