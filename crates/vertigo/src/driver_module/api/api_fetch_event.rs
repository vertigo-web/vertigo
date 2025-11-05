use std::rc::Rc;

use crate::driver_module::event_emitter::EventEmitter;


pub struct ApiFetchEvent {
    pub on_fetch_start: EventEmitter<()>,
    pub on_fetch_stop: EventEmitter<()>,
}

impl  ApiFetchEvent {
    fn new() -> ApiFetchEvent {
        ApiFetchEvent {
            on_fetch_start: EventEmitter::default(),
            on_fetch_stop: EventEmitter::default(),
        }
    }
}

use vertigo_macro::store;

#[store]
pub fn api_fetch_event() -> Rc<ApiFetchEvent> {
    Rc::new(ApiFetchEvent::new())
}

