use web_sys::{Event, /* MouseEvent, KeyboardEvent,*/ EventTarget};
use wasm_bindgen::{JsCast, prelude::Closure};

use alloc::boxed::Box;

pub struct DomEvent {
    closure: Closure<dyn Fn(Event)>,
}

impl DomEvent {
    fn new<F: Fn(Event) + 'static>(func: F) -> DomEvent {
        let func_boxed = Box::new(func);

        let closure: Closure<dyn Fn(Event)> = Closure::new(move |event: Event| {
            func_boxed(event)
        });

        DomEvent {
            closure
        }
    }

    fn append_to(self, event_name: &'static str, target: &EventTarget) -> DomEventDisconnect {
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

    pub fn new_event<F: Fn(Event) + 'static>(target: &EventTarget, event_name: &'static str, func: F) -> DomEventDisconnect {
        DomEvent::new(func).append_to(event_name, target)
    }
}

pub struct DomEventDisconnect {
    target: EventTarget,
    event_name: &'static str,
    closure: Closure<dyn Fn(Event)>,
}

impl Drop for DomEventDisconnect {
    fn drop(&mut self) {
        self.target.remove_event_listener_with_callback(
            self.event_name,
            self.closure.as_ref().unchecked_ref()
        ).unwrap();
    }
}

// pub struct DomEventMouse {
//     event: DomEvent,
// }

// impl DomEventMouse {
//     pub fn new<F: FnMut(&MouseEvent) + 'static>(func: F) -> DomEventMouse {
//         let mut func_boxed = Box::new(func);

//         let event = DomEvent::new(move |event: Event| {
//             let event_mouse = event.dyn_ref::<MouseEvent>().unwrap();
//             func_boxed(event_mouse);
//         });

//         DomEventMouse {
//             event
//         }
//     }

//     pub fn new_event<F: FnMut(&MouseEvent) + 'static>(target: &EventTarget, event_name: &'static str, func: F) -> DomEventDisconnect {
//         DomEventMouse::new(func).event.append_to(event_name, target)
//     }
// }


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

//     pub fn new_event<F: FnMut(&KeyboardEvent) + 'static>(target: &EventTarget, event_name: &'static str, func: F) -> DomEventDisconnect {
//         DomEventKeyboard::new(func).event.append_to(event_name, target)
//     }
// }


