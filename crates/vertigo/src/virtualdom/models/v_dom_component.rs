use std::cmp::PartialEq;
use std::rc::Rc;
use crate::computed::{
    Value,
    Computed,
};

use crate::{
    virtualdom::{
        models::{
            v_dom_node::VDomNode,
            v_dom_component_id::VDomComponentId,
        }
    }
};

#[derive(Clone)]
pub struct VDomComponent {
    pub id: VDomComponentId,
    pub view: Computed<VDomNode>,
}

impl VDomComponent {
    pub fn new<T: PartialEq + 'static>(params: Computed<T>, render: fn(&Computed<T>) -> VDomNode) -> VDomComponent {

        let component_id = VDomComponentId::new(&params, render);
        let view = params.map_for_render(render);

        VDomComponent {
            id: component_id,
            view,
        }
    }

    pub fn from_value<T: PartialEq + 'static>(params: Value<T>, render: fn(&Value<T>) -> VDomNode) -> VDomComponent {

        let component_id = VDomComponentId::new_value(&params, render);

        let deps = params.deps.clone();

        let comp = deps.new_computed_from(params);

        let view = comp.map(move |wrapper_computed: &Computed<Value<T>>| -> Rc<VDomNode> {
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
