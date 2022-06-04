use std::{
    fmt,
    rc::Rc, any::Any,
};

use crate::{
    virtualdom::{models::vdom_element::VDomElement, render_to_node::update_node},
    Computed, get_driver, Context
};

use super::{dom_node::DomElement, vdom_component_id::VDomComponentId};

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

#[derive(Clone)]
enum Render {
    VDom(Rc<dyn RenderVDom>),                               //Virtual DOM
    BuildRealDom(DomElement),           //TODO FnOnce(RealDomElement) -> RealDomElement ?
}

#[derive(Clone)]
pub struct VDomComponent {
    id: VDomComponentId,
    render: Render,
}

impl VDomComponent {
    pub fn id(&self) -> VDomComponentId {
        self.id
    }

    pub fn from<T: 'static>(state: T, render: impl Fn(&Context, &T) -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: VDomComponentId::default(),
            render: Render::VDom(Rc::new(VDomComponentRender::new(state, move |state: &T| {
                let context = Context::new();
                render(&context, state)
            }))),
        }
    }

    pub fn from_ref<T: Clone + 'static>(state: &T, render: impl Fn(&Context, &T) -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: VDomComponentId::default(),
            render: Render::VDom(Rc::new(VDomComponentRender::new(state.clone(), move |state: &T| {
                let context = Context::new();
                render(&context, state)
            }))),
        }
    }

    pub fn from_fn(render: impl Fn(&Context) -> VDomElement + 'static) -> VDomComponent {
        VDomComponent {
            id: VDomComponentId::default(),
            render: Render::VDom(Rc::new(VDomFunction {
                render: Box::new(move ||{
                    let context = Context::new();
                    render(&context)
                })
            }))
        }
    }

    pub fn dom(build: DomElement) -> VDomComponent {
        VDomComponent {
            id: VDomComponentId::default(),
            render: Render::BuildRealDom(build)
        }
    }

    pub fn render_to(self, target: DomElement) -> Box<dyn Any> {
        let Self { id: _, render } = self;

        match render {
            Render::VDom(render) => {
                let view = Computed::from(move |_| {
                    let dom_element = render.render();
                    Rc::new(dom_element)
                });

                let client = view.subscribe(move |new_version| {
                    update_node(&target, new_version.as_ref());
                });

                Box::new(client)
            },
            Render::BuildRealDom(build) => {

                get_driver().get_dependencies().block_tracking_on();
                
                let dom = build;
                let target = target.child(dom);

                get_driver().get_dependencies().block_tracking_off();

                Box::new(target)
            }
        }
    }

}

impl fmt::Debug for VDomComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VDomComponent")
            .field("id", &self.id)
            .finish()
    }
}
