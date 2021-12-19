use crate::{
    computed::{Computed, GraphId, Value},
    virtualdom::models::vdom_element::VDomElement,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VDomComponentId {
    computed_id: GraphId, // id of the particular computed
    function_id: u64,     // id of the particular static function (that renders the component)
}

impl VDomComponentId {
    pub fn new<T: PartialEq>(params: &Computed<T>, render: fn(&Computed<T>) -> VDomElement) -> VDomComponentId {
        let function_id = render as *const () as u64;
        VDomComponentId {
            computed_id: params.get_id(),
            function_id,
        }
    }

    pub fn new_value<T: PartialEq>(params: &Value<T>, render: fn(&Value<T>) -> VDomElement) -> VDomComponentId {
        let function_id = render as *const () as u64;
        VDomComponentId {
            computed_id: params.id(),
            function_id,
        }
    }
}
