use vertigo::{Computed, Driver, VDomElement, dev::start_app};

/// Start application using driver browser.
///
/// Given the state factory and main render function, it creates necessary vertigo facilities
/// and runs a never-ending future of reactivity.
///
/// ```rust
/// use vertigo::{html, Computed, Driver, VDomElement, Value};
/// use vertigo_browserdriver::prelude::*;
///
/// #[derive(PartialEq)]
/// pub struct State;
///
/// impl State {
///     pub fn new(driver: &Driver) -> Computed<State> {
///         driver.new_computed_from(State)
///     }
/// }
///
/// pub fn render(_state: &Computed<State>) -> VDomElement {
///     html! {
///         <div>"Hello world"</div>
///     }
/// }
///
/// #[wasm_bindgen_derive(start)]
/// pub async fn start_application() {
///     let driver = DriverBrowser::new();
///     let state = State::new(&driver);
///     start_browser_app(driver, state, render).await;
/// }
/// ```
pub async fn start_browser_app<T: PartialEq + 'static>(driver: Driver, state: Computed<T>, render: fn(&Computed<T>) -> VDomElement) {
    #[cfg(feature = "wasm_logger")]
    {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());
    }

    start_app(driver, state, render).await
}
