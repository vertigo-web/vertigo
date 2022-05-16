use std::{
    fmt,
    rc::Rc,
};

use crate::{
    virtualdom::models::vdom_element::VDomElement,
    GraphId
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

struct VDomFunction {
    render: Box<dyn Fn() -> VDomElement>,
}

impl RenderVDom for VDomFunction {
    fn render(&self) -> VDomElement {
        let render = &self.render;
        render()
    }
}

/// A component is a virtual dom element with render function attached to it.
///
/// Usually used as a main component for the application.
///
/// ```rust
/// use vertigo::{Computed, Value, VDomComponent, VDomElement, html};
///
/// let state = Value::new(5);
///
/// fn comp_render(state: &Value<i32>) -> VDomElement {
///     html! { <p>{*state.get_value()}</p> }
/// }
///
/// let main_component = VDomComponent::from(state, comp_render);
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

    pub fn from<T: 'static>(state: T, render: impl Fn(&T) -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: GraphId::default(),
            render: Rc::new(VDomComponentRender::new(state, render)),
        }
    }

    pub fn from_ref<T: Clone + 'static>(state: &T, render: impl Fn(&T) -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: GraphId::default(),
            render: Rc::new(VDomComponentRender::new(state.clone(), render)),
        }
    }

    pub fn from_fn(render: impl Fn() -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: GraphId::default(),
            render: Rc::new(VDomFunction {
                render: Box::new(render)
            })
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
