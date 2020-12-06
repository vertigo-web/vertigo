use std::rc::Rc;

use crate::{
    computed::{
        Computed::Computed,
        Dependencies::Dependencies,
        //Value::Value
    },
    vdom::models::{
        VDom::VDom,
        VDomComponent::VDomComponent,
    }
};

pub struct StateBox<T: 'static> {
    computed: Computed<Rc<T>>,
}

impl<T: 'static> StateBox<T> {
    pub fn new(root: &Dependencies, state: T) -> StateBox<T> {
        let state = root.newValue(Rc::new(state));
        let computed = state.toComputed();

        StateBox {
            computed,
        }
    }

    pub fn render(&self, render: fn(&Rc<T>) -> Vec<VDom>) -> VDomComponent {
        VDomComponent::new(&self.computed, render)
    }

    pub fn toComputed(&self) -> Computed<Rc<T>> {
        self.computed.clone()
    }
}
