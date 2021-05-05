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
pub mod keydown;

pub fn get_from_node<R>(inner: &Rc<BoxRefCell<DomDriverBrowserInner>>, node_id: &RealDomId, map: fn(&ElementWrapper) -> Option<R>) -> Option<R> {
    inner.get_with_context((node_id, map), |state, (node_id, map)| {
        match state.elements.get(node_id) {
            Some(element) => map(element),
            None => {
                log::error!("get_from_node - missing node {}", node_id);
                None
            }
        }
    })
}


pub fn find_all_nodes(
    inner: &Rc<BoxRefCell<DomDriverBrowserInner>>,
    id: RealDomId,
) -> Vec<RealDomId> {
    inner.get_with_context(
        id,
        |state, id| -> Vec<RealDomId> {
            if id == RealDomId::root() {
                return vec![RealDomId::root()];
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
                        out.push(parent.clone());
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

fn find_dom_id_option(event: &web_sys::Event) -> Option<RealDomId> {
    let target = event.target().unwrap();
    let element = target.dyn_ref::<Element>().unwrap();

    let option_id: Option<String> = (*element).get_attribute("data-id");
    let option_id = match option_id {
        Some(option_id) => option_id,
        None => {
            return None;
        }
    };

    let id = option_id.parse::<u64>();

    match id {
        Ok(id) => Some(RealDomId::from_u64(id)),
        Err(_) => None
    }
}

fn find_dom_id(event: &web_sys::Event) -> RealDomId {
    find_dom_id_option(event).unwrap()
}
