use std::rc::Rc;
use std::fmt::Debug;

use crate::computed::{
    BoxRefCell,
    Dependencies,
    Client,
    refresh_token::RefreshToken,
    graph_id::GraphId,
};




pub struct ComputedInner<T: 'static> {
    deps: Dependencies,
    get_value_from_parent: Box<dyn Fn() -> Rc<T> + 'static>,
    id: GraphId,
    is_fresh_cell: Rc<BoxRefCell<bool>>,
    value_cell: BoxRefCell<Rc<T>>,
}

impl<T> Drop for ComputedInner<T> {
    fn drop(&mut self) {
        self.deps.remove_relation(&self.id);
    }
}



pub struct Computed<T: 'static> {
    inner: Rc<ComputedInner<T>>,
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone()
        }
    }
}

impl<T: 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Dependencies, get_value: F) -> Computed<T> {

        let id = GraphId::default();
        let is_fresh_cell = Rc::new(BoxRefCell::new(true));

        let get_value = deps.wrap_get_value(get_value, id.clone());

        deps.register_refresh_token(id.clone(), RefreshToken::new_computed(is_fresh_cell.clone()));

        let value = get_value();

        Computed {
            inner: Rc::new(ComputedInner {
                deps,
                get_value_from_parent: get_value,
                id,
                is_fresh_cell: is_fresh_cell,
                value_cell: BoxRefCell::new(value),
            })
        }
    }

    pub fn get_id(&self) -> GraphId {
        self.inner.id.clone()
    }

    pub fn get_value(&self) -> Rc<T> {
        let inner = self.inner.as_ref();
        let self_id = inner.id.clone();
        let deps = inner.deps.clone();

        deps.report_dependence_in_stack(self_id);

        let should_recalculate = {
            self.inner.is_fresh_cell.change_no_params(|state|{
                let should_recalculate = !(*state);
                *state = true;
                should_recalculate
            })
        };

        let new_value = if should_recalculate {
            let ComputedInner { get_value_from_parent, .. } = self.inner.as_ref();
            let result = get_value_from_parent();
            Some(result)
        } else {
            None
        };

        inner.value_cell.change(new_value, |state, new_value| {
            if let Some(value) = new_value {
                *state = value;
            }

            (*state).clone()
        })
    }

    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        Client::new(self.inner.deps.clone(), self.clone(), call)
    }

    pub fn dependencies(&self) -> Dependencies {
        self.inner.deps.clone()
    }

    pub fn from2<A: Debug, B: Debug>(
        a: Computed<A>,
        b: Computed<B>,
        calculate: fn(Rc<A>, Rc<B>) -> T
    ) -> Computed<T> {
        let deps = a.inner.deps.clone();

        Computed::new(deps, move || {
            let a_value = a.get_value();
            let b_value = b.get_value();

            let result = calculate(a_value, b_value);

            Rc::new(result)
        })
    }

    pub fn map_for_render<K>(self, fun: fn(&Computed<T>) -> K) -> Computed<K> {
        let deps = self.inner.deps.clone();

        Computed::new(deps, move || {
            let result = fun(&self);
            Rc::new(result)
        })
    }

    pub fn map<K, F: 'static + Fn(&Computed<T>) -> Rc<K>>(self, fun: F) -> Computed<K> {
        let deps = self.inner.deps.clone();

        Computed::new(deps, move || {
            let result = fun(&self);
            result
        })
    }
}
