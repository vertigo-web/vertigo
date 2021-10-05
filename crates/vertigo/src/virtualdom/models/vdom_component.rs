use std::cmp::PartialEq;
use std::rc::Rc;
use std::fmt;

use crate::computed::{
    Value,
    Computed,
};

use crate::{
    virtualdom::{
        models::{
            vdom_component_id::VDomComponentId,
            vdom_element::VDomElement,
        }
    }
};

#[derive(Clone)]
pub struct VDomComponent {
    pub id: VDomComponentId,
    pub view: Computed<VDomElement>,
}

impl VDomComponent {
    pub fn new<T: PartialEq + 'static>(params: Computed<T>, render: fn(&Computed<T>) -> VDomElement) -> VDomComponent {

        let component_id = VDomComponentId::new(&params, render);
        let view = params.map_for_render(render);

        VDomComponent {
            id: component_id,
            view,
        }
    }

    pub fn from_value<T: PartialEq + 'static>(params: Value<T>, render: fn(&Value<T>) -> VDomElement) -> VDomComponent {

        let component_id = VDomComponentId::new_value(&params, render);

        let deps = params.deps.clone();

        let comp = deps.new_computed_from(params);

        let view = comp.map(move |wrapper_computed: &Computed<Value<T>>| -> VDomElement {
            let value: Rc<Value<T>> = wrapper_computed.get_value();
            let value: &Value<T> = value.as_ref();
            render(value)
        });

        VDomComponent {
            id: component_id,
            view,
        }
    }
}

impl fmt::Debug for VDomComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VDomElement")
            .field("id", &self.id)
            .field("view", &self.view.get_value())
            .finish()
    }
}
