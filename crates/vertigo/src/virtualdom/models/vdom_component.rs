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

/// A component is a virtual dom element with render function attached to it.
///
/// Usually used as a main component for the application.
///
/// ```rust,no_run
/// use vertigo::{Computed, VDomComponent, VDomElement, Dependencies, html};
///
/// // Here some driver should be used instead of pure dependency graph.
/// let deps = Dependencies::default();
///
/// let state = deps.new_computed_from(5);
///
/// fn comp_render(state: &Computed<i32>) -> VDomElement {
///     html! { <p>{*state.get_value()}</p> }
/// }
///
/// let main_component = VDomComponent::new(state, comp_render);
/// ```
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
