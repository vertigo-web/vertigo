use web_sys::{Event, EventTarget};
use wasm_bindgen::{JsCast, prelude::Closure};

pub struct DomEvent {
    target: EventTarget,
    event_name: &'static str,
    closure: Closure<dyn Fn(Event)>,
}

impl DomEvent {
    pub fn new<F: Fn(Event) + 'static>(target: &EventTarget, event_name: &'static str, func: F) -> DomEvent {
        let func_boxed = Box::new(func);

        let closure: Closure<dyn Fn(Event)> = Closure::new(move |event: Event| {
            func_boxed(event)
        });

        target.add_event_listener_with_callback(
            event_name,
            closure.as_ref().unchecked_ref()
        ).unwrap();

        DomEvent {
            target: target.clone(),
            event_name,
            closure
        }
    }
}

impl Drop for DomEvent {
    fn drop(&mut self) {
        self.target.remove_event_listener_with_callback(
            self.event_name,
            self.closure.as_ref().unchecked_ref()
        ).unwrap();
    }
}
