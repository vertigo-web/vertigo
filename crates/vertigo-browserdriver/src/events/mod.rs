use std::{
    rc::Rc,
};
use wasm_bindgen::{JsCast};
use web_sys::{Element};

use vertigo::{
    RealDomId,
    utils::{
        BoxRefCell,
    }
};

use crate::{DomDriverBrowserInner, element_wrapper::ElementWrapper};

pub mod input;
pub mod mousedown;
pub mod mouseenter;

fn find_event<T: Clone>(
    inner: &Rc<BoxRefCell<DomDriverBrowserInner>>,
    id: RealDomId,
    find_event_on_click_item: fn(&ElementWrapper) -> &Option<T>,
) -> Option<T> {
    let on_click = inner.get_with_context(
        (id, find_event_on_click_item),
        |state, (id, find_event)| -> Option<T> {
            let mut wsk = id;
            let mut count = 0;

            loop {
                count += 1;

                if count > 100 {
                    log::error!("Too many nested levels");
                    return None;
                }

                let item = state.elements.get(&wsk).unwrap();

                let item_inner = find_event(item);

                if let Some(on_click) = item_inner {
                    return Some(on_click.clone());
                }

                let parent = state.child_parent.get(&wsk);
                if let Some(parent) = parent {
                    wsk = parent.clone();
                } else {
                    return None;
                }
            }
        }
    );

    on_click
}


pub fn find_all_nodes(
    inner: &Rc<BoxRefCell<DomDriverBrowserInner>>,
    id: RealDomId,
) -> Vec<RealDomId> {
    inner.get_with_context(
        id,
        |state, id| -> Vec<RealDomId> {
            if id == RealDomId::root() {
                return Vec::new();
            }
            
            let mut wsk = id.clone();
            let mut count = 0;
            let mut out: Vec<RealDomId> = Vec::new();

            loop {
                out.push(wsk.clone());

                count += 1;

                if count > 100 {
                    log::error!("Too many nested levels");
                    return out;
                }

                let parent = state.child_parent.get(&wsk);
                if let Some(parent) = parent {
                    if *parent == RealDomId::root() {
                        return out;
                    } else {
                        wsk = parent.clone();
                    }
                } else {
                    log::error!("It should never have happened {:?}", id);
                    return out;
                }
            }
        }
    )
}

fn find_dom_id(event: &web_sys::Event) -> RealDomId {
    let target = event.target().unwrap();
    let element = target.dyn_ref::<Element>().unwrap();

    let option_id: Option<String> = (*element).get_attribute("data-id");
    let id: u64 = option_id.unwrap().parse::<u64>().unwrap();
    RealDomId::from_u64(id)
}
