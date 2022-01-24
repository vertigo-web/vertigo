use std::rc::Rc;

use crate::{
    computed::{Client, Value},
    Driver,
    utils::{DropResource}, struct_mut::ValueMut,
};

#[derive(PartialEq)]
pub struct HashRouter {
    sender: Client,
    receiver: DropResource,
}

/// Router based on hash part of current location.
///
/// ```rust
/// use vertigo::{html, Computed, Driver, Value, VDomElement, VDomComponent};
/// use vertigo::router::HashRouter;
///
/// #[derive(PartialEq, Debug)]
/// pub enum Route {
///     Page1,
///     Page2,
///     NotFound,
/// }
///
/// impl Route {
///     pub fn new(path: &str) -> Route {
///         match path {
///             "" | "/" | "/page1" => Self::Page1,
///             "page2" => Self::Page2,
///             _ => Self::NotFound,
///         }
///     }
/// }
///
/// impl ToString for Route {
///     fn to_string(&self) -> String {
///         match self {
///             Self::Page1 => "",
///             Self::Page2 => "page2",
///             Self::NotFound => "",
///         }.to_string()
///     }
/// }
///
/// #[derive(PartialEq)]
/// pub struct State {
///     pub driver: Driver,
///     pub route: Value<Route>,
///
///     hash_router: HashRouter,
/// }
///
/// impl State {
///     pub fn component(driver: &Driver) -> VDomComponent {
///         let route: Value<Route> = driver.new_value(Route::new(&driver.get_hash_location()));
///
///         let hash_router = HashRouter::new(driver, route.clone(), {
///             let route = route.clone();
///
///             Box::new(move |url: &String|{
///                 route.set_value(Route::new(url));
///             })
///         });
///
///         let state = State {
///             driver: driver.clone(),
///             route,
///             hash_router,
///         };
///
///         driver.bind_render(state, render)
///     }
/// }
/// 
/// fn render(state: &Computed<State>) -> VDomElement {
///     html! {
///         <div>
///             "..."
///         </div>
///     }
/// }
/// ```
impl HashRouter {
    /// Create new HashRouter which sets route value upon hash change in browser bar.
    /// If callback is provided then it is fired instead.
    pub fn new<T>(driver: &Driver, route: Value<T>, callback: Box<dyn Fn(&String)>) -> Self
    where
        T: PartialEq + ToString,
    {
        let block_subscrition = Rc::new(ValueMut::new(true));

        let sender = route.to_computed().subscribe({
            let driver = driver.clone();
            let block_subscrition = block_subscrition.clone();
            move |route| {
                if block_subscrition.get() {
                    return;
                }

                driver.push_hash_location(route.to_string());
            }
        });

        let receiver = driver.on_hash_route_change({
            let driver = driver.clone();
            let block_subscrition = block_subscrition.clone();

            Box::new(move |url: &String| {
                block_subscrition.set(true);
                driver.transaction(|| {
                    callback(url);
                });
                block_subscrition.set(false);
            })
        });

        block_subscrition.set(false);

        Self { sender, receiver }
    }
}
