use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::prelude::Closure;

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn clearInterval(token: f64);
}

#[wasm_bindgen]
pub struct Interval {
    _closure: Closure<dyn FnMut()>,
    token: f64,
}

impl Interval {
    pub fn new(millis: u32, func: Box<dyn Fn()>) -> Interval {
        // Construct a new closure.
        let closure = Closure::new(func);

        // Pass the closure to JS, to run every n milliseconds.
        let token = setInterval(&closure, millis);

        Interval {
            _closure: closure,
            token
        }
    }

    pub fn off(self) {}
}

// When the Interval is destroyed, cancel its `setInterval` timer.
impl Drop for Interval {
    fn drop(&mut self) {
        clearInterval(self.token);
    }
}

// Keep logging "hello" every second until the resulting `Interval` is dropped.

// pub fn hello_interval() -> Interval {
//     Interval::new(1_000, Box::new(|| log::info!("hello")))
// }