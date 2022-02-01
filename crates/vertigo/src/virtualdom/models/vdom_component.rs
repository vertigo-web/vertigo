use std::{
    fmt,
    rc::Rc,
};

use crate::{
    virtualdom::models::{
        vdom_element::VDomElement
    }, GraphId
};

pub trait RenderVDom {
    fn render(&self) -> VDomElement;
}

struct VDomComponentRender<T> {
    state: T,
    render: Box<dyn Fn(&T) -> VDomElement>,
}

impl<T> VDomComponentRender<T> {
    pub fn new(state: T, render: impl Fn(&T) -> VDomElement + 'static) -> VDomComponentRender<T> {
        VDomComponentRender {
            state,
            render: Box::new(render),
        }
    }
}

impl<T> RenderVDom for VDomComponentRender<T> {
    fn render(&self) -> VDomElement {
        let state = &self.state;
        let render = &self.render;

        render(state)
    }
}

/// A component is a virtual dom element with render function attached to it.
///
/// Usually used as a main component for the application.
///
/// ```rust,no_run
/// use vertigo::{Computed, Dependencies, VDomComponent, VDomElement, html};
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
    id: GraphId,
    pub render: Rc<dyn RenderVDom>,
}

impl VDomComponent {
    pub fn id(&self) -> GraphId {
        self.id
    }

    pub fn new<T: 'static>(state: T, render: impl Fn(&T) -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: GraphId::default(),
            render: Rc::new(VDomComponentRender::new(state, render)),
        }
    }
}

impl fmt::Debug for VDomComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VDomComponent")
            .field("id", &self.id)
            .field("render", &(self.render.as_ref() as *const dyn RenderVDom))
            .finish()
    }
}
