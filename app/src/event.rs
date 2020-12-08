use web_sys::EventTarget;
use wasm_bindgen::{JsCast, prelude::Closure};

// let closure: Closure<dyn FnMut(_)> = {
//     let val1 = val1.clone();

//     Closure::new(move |event: web_sys::Event| {
//         log::info!("click ...");
//         // let target = event.related_target();

//         let target2 = event.target();

//         if let Some(target) = target2 {
//             let ta  = target.dyn_ref::<web_sys::Element>().unwrap();
//             log::info!("sprawdzam target {}", ta /* as web_sys::Element*/ == &val1);
//         } else {
//             log::info!("brak targeta");
//         }


//         let kon = event.dyn_ref::<web_sys::MouseEvent>();
//         log::info!("skonwertowanie na event myszy {:?}", kon);
//     })
// };

// (&body).add_event_listener_with_callback(
//     "mousedown",
//     closure.as_ref().unchecked_ref()
// ).unwrap();

// closure.forget();

pub struct DomEvent {
    closure: Closure<dyn FnMut(web_sys::Event)>,
}

impl DomEvent {
    pub fn new<F: FnMut(web_sys::Event) + 'static>(func: F) -> DomEvent {
        let mut func_boxed = Box::new(func);

        let closure: Closure<dyn FnMut(web_sys::Event)> = Closure::new(move |event: web_sys::Event| {
            func_boxed(event)
        });

        DomEvent {
            closure: closure
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
    closure: Closure<dyn FnMut(web_sys::Event)>,
}

impl Drop for DomEventDisconnect {
    fn drop(&mut self) {
        self.target.remove_event_listener_with_callback(
            self.event_name,
            self.closure.as_ref().unchecked_ref()
        ).unwrap();
    }
}

