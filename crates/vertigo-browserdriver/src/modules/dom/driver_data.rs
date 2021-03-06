use std::rc::Rc;

use vertigo::{
    KeyDownEvent,
    RealDomId,
};

use crate::utils::hash_map_rc::HashMapRc;

use super::element_wrapper::{DomElement, DomText};

pub struct DriverData {
    pub elements: HashMapRc<RealDomId, DomElement>,
    pub texts: HashMapRc<RealDomId, DomText>,
    pub child_parent: HashMapRc<RealDomId, RealDomId>,            //child -> parent
}

impl DriverData {
    pub fn new() -> Rc<DriverData> {
        Rc::new(DriverData {
            elements: HashMapRc::new("DriverData elements"),
            texts: HashMapRc::new("DriverData texts"),
            child_parent: HashMapRc::new("DriverData child_parent"),
        })
    }

    pub fn find_all_nodes(
        &self,
        id: RealDomId,
    ) -> Vec<RealDomId> {
        if id == RealDomId::root() {
            return vec![RealDomId::root()];
        }

        let mut wsk = id.clone();
        let mut count = 0;
        let mut out: Vec<RealDomId> = vec![wsk.clone()];

        loop {
            count += 1;

            if count > 100 {
                log::error!("Too many nested levels");
                return out;
            }

            let parent = self.child_parent.must_get(
                &wsk,
                |item| item.clone()
            );

            if let Some(parent) = parent {
                out.push(parent.clone());

                if parent == RealDomId::root() {    
                    return out;
                } else {
                    wsk = parent;
                }
            } else {
                log::error!("It should never have happened {:?}", id);
                return out;
            }
        }
    }

    pub fn get_from_node<R>(&self, node_id: &RealDomId, map: fn(&DomElement) -> Option<R>) -> Option<R> {
        self.elements.must_get(node_id, map).flatten()
    }


    pub fn find_event_click(
        &self,
        id: RealDomId,
    ) -> Option<Rc<dyn Fn()>> {

        let all_nodes = self.find_all_nodes(id);

        for node_id in all_nodes {

            let on_click = self.get_from_node(
                &node_id,
                |elem| elem.on_click.clone()
            );

            if on_click.is_some() {
                return on_click;
            }
        }

        None
    }

    pub fn find_event_keydown(
        &self,
        id: RealDomId,
    ) -> Option<Rc<dyn Fn(KeyDownEvent) -> bool>> {
        let all_nodes = self.find_all_nodes(id);

        for node_id in all_nodes {
            let on_input = self.get_from_node(
                &node_id,
                |elem| elem.on_input.clone()
            );
                                    //cancel the bubbling event
            if on_input.is_some() {
                return None;
            }

            let on_key = self.get_from_node(
                &node_id,
                |elem| elem.on_keydown.clone()
            );

            if on_key.is_some() {
                return on_key;
            }
        }

        None
    }

    pub fn find_event_on_input(&self, id: RealDomId) -> Option<Rc<dyn Fn(String)>> {
        self.get_from_node(
            &id,
            |elem| elem.on_input.clone()
        )
    }
}
