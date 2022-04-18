use std::rc::Rc;

use crate::{
    computed::{Client, Value, DropResource},
    Driver,
    struct_mut::ValueMut,
};

struct HashSubscriptions {
    _sender: Client,
    _receiver: DropResource,
}

#[derive(Clone)]
pub struct HashRouter<T: ToString + From<String> + PartialEq + 'static> {
    route: Value<T>,
    _subscriptions: Rc<HashSubscriptions>,
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
/// impl From<String> for Route {
///     fn from(url: String) -> Self {
///         Route::new(url.as_str())
///     }
/// }
///
/// pub struct State {
///     pub driver: Driver,
///     route: HashRouter<Route>,
/// }
///
/// impl State {
///     pub fn component(driver: &Driver) -> VDomComponent {
///         let route = HashRouter::new(driver);
///
///         let state = State {
///             driver: driver.clone(),
///             route,
///         };
///
///         VDomComponent::new(state, render)
///     }
/// }
/// 
/// fn render(state: &State) -> VDomElement {
///     html! {
///         <div>
///             "..."
///         </div>
///     }
/// }
/// ```
impl<T: ToString + From<String> + PartialEq + 'static> HashRouter<T> {
    /// Create new HashRouter which sets route value upon hash change in browser bar.
    /// If callback is provided then it is fired instead.
    pub fn new(driver: &Driver) -> Self {
        let route: Value<T> = driver.new_value(T::from(driver.get_hash_location()));

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
            let route = route.clone();
            let block_subscrition = block_subscrition.clone();

            Box::new(move |url: &String| {
                block_subscrition.set(true);
                route.set_value_and_compare(T::from(url.clone()));
                block_subscrition.set(false);
            })
        });

        block_subscrition.set(false);

        Self {
            route,
            _subscriptions: Rc::new(HashSubscriptions {
                _sender: sender,
                _receiver: receiver
            })
        }
    }

    pub fn get_value(&self) -> Rc<T> {
        self.route.get_value()
    }

    pub fn set_value(&self, value: T) {
        self.route.set_value_and_compare(value);
    }
}
