use std::cmp::PartialEq;
use std::rc::Rc;
use crate::computed::{
    Value,
    Computed,
};

use crate::{
    virtualdom::{
        models::{
            vdom_node::VDomElement,
            vdom_component_id::VDomComponentId,
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

        let view = comp.map(move |wrapper_computed: &Computed<Value<T>>| -> Rc<VDomElement> {
            let value: Rc<Value<T>> = wrapper_computed.get_value();
            let value: &Value<T> = value.as_ref();
            Rc::new(render(value))
        });

        
        VDomComponent {
            id: component_id,
            view,
        }
    }
}
