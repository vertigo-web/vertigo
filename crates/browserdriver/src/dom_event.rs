use web_sys::{Event, MouseEvent, /*KeyboardEvent,*/ EventTarget};
use wasm_bindgen::{JsCast, prelude::Closure};

pub struct DomEvent {
    closure: Closure<dyn FnMut(Event)>,
}

impl DomEvent {
    pub fn new<F: FnMut(Event) + 'static>(func: F) -> DomEvent {
        let mut func_boxed = Box::new(func);

        let closure: Closure<dyn FnMut(Event)> = Closure::new(move |event: Event| {
            func_boxed(event)
        });

        DomEvent {
            closure
        }
    }

    pub fn append_to(self, event_name: &'static str, target: &EventTarget) -> DomEventDisconnect {
        target.add_event_listener_with_callback(
            event_name,
            self.closure.as_ref().unchecked_ref()
        ).unwrap();

        DomEventDisconnect {
            target: target.clone(),
            event_name,
            closure: self.closure,
        }
    }
}

pub struct DomEventDisconnect {
    target: EventTarget,
    event_name: &'static str,
    closure: Closure<dyn FnMut(Event)>,
}

impl Drop for DomEventDisconnect {
    fn drop(&mut self) {
        self.target.remove_event_listener_with_callback(
            self.event_name,
            self.closure.as_ref().unchecked_ref()
        ).unwrap();
    }
}

pub struct DomEventMouse {
    event: DomEvent,
}

impl DomEventMouse {
    pub fn new<F: FnMut(&MouseEvent) + 'static>(func: F) -> DomEventMouse {
        let mut func_boxed = Box::new(func);

        let event = DomEvent::new(move |event: Event| {
            let event_mouse = event.dyn_ref::<MouseEvent>().unwrap();
            func_boxed(event_mouse);
        });

        DomEventMouse {
            event
        }
    }

    pub fn append_to_mousedown(self, target: &EventTarget) -> DomEventDisconnect {
        self.event.append_to("mousedown", target)
    }
}


// pub struct DomEventKeyboard {
//     event: DomEvent,
// }

// impl DomEventKeyboard {
//     pub fn new<F: FnMut(&KeyboardEvent) + 'static>(func: F) -> DomEventKeyboard {
//         let mut func_boxed = Box::new(func);

//         let event = DomEvent::new(move |event: Event| {
//             let event_keyboard = event.dyn_ref::<KeyboardEvent>().unwrap();
//             func_boxed(event_keyboard);
//         });

//         DomEventKeyboard {
//             event
//         }
//     }

//     pub fn append_to_keydown(self, target: &EventTarget) -> DomEventDisconnect {
//         self.event.append_to("keydown", target)
//     }
// }


