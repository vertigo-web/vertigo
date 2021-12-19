use wasm_bindgen::prelude::{wasm_bindgen, Closure};

#[wasm_bindgen(module = "/src/modules/websocket/js_websocket.js")]
extern "C" {
    pub type DriverWebsocketJs;

    #[wasm_bindgen(constructor)]
    pub fn new(
        callback_socket: &Closure<dyn Fn(u64)>,
        callback_message: &Closure<dyn Fn(u64, String)>,
        callback_close: &Closure<dyn Fn(u64)>,
    ) -> DriverWebsocketJs;

    #[wasm_bindgen(method)]
    pub fn register_callback(this: &DriverWebsocketJs, host: String, callback_id: u64);

    #[wasm_bindgen(method)]
    pub fn unregister_callback(this: &DriverWebsocketJs, callback_id: u64);

    #[wasm_bindgen(method)]
    pub fn send_message(this: &DriverWebsocketJs, callback_id: u64, message: String);
}
