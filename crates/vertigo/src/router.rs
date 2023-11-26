use crate::{
    computed::Value,
    get_driver,
    Computed,
};

/// Router based on hash part of current location.
///
/// ```rust
/// use vertigo::{dom, Computed, Value, DomNode};
/// use vertigo::router::Router;
///
/// #[derive(Clone, PartialEq, Debug)]
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
///             "/page2" => Self::Page2,
///             _ => Self::NotFound,
///         }
///     }
/// }
///
/// impl ToString for Route {
///     fn to_string(&self) -> String {
///         match self {
///             Self::Page1 => "/",
///             Self::Page2 => "/page2",
///             Self::NotFound => "/404",
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
/// #[derive(Clone)]
/// pub struct State {
///     route: Router<Route>,
/// }
///
/// impl State {
///     pub fn component() -> DomNode {
///         let route = Router::new_history_router();
///
///         let state = State {
///             route,
///         };
///
///         render(state)
///     }
/// }
///
/// fn render(state: State) -> DomNode {
///     dom! {
///         <div>
///             "..."
///         </div>
///     }
/// }
/// ```
#[derive(Clone, PartialEq)]
pub struct Router<T: Clone + ToString + From<String> + PartialEq + 'static> {
    use_history_api: bool,
    pub route: Computed<T>,
}

impl<T: Clone + ToString + From<String> + PartialEq + 'static> Router<T> {
    /// Create new Router which sets route value upon hash change in browser bar.
    /// If callback is provided then it is fired instead.
    pub fn new_hash_router() -> Router<T> {
        Self::new(false)
    }

    /// Create new Router which sets route value upon url change (works with browser history)
    pub fn new_history_router() -> Router<T> {
        Self::new(true)
    }

    fn new(use_history_api: bool) -> Self {
        let driver = get_driver();

        let init_value = match use_history_api {
            false => T::from(driver.inner.api.get_hash_location()),
            true => T::from(driver.inner.api.get_history_location()),
        };

        let route = Value::with_connect(init_value, move |value| {
            let value = value.clone();
            let callback = move |url: String| {
                value.set_value_and_compare(T::from(url));
            };

            match use_history_api {
                false => driver.inner.api.on_hash_change(callback),
                true => driver.inner.api.on_history_change(callback),
            }
        });

        Self {
            use_history_api,
            route,
        }
    }

    pub fn set(&self, route: T) {
        let driver = get_driver();
        match self.use_history_api {
            false => driver.inner.api.push_hash_location(&route.to_string()),
            true => driver.inner.api.push_history_location(&route.to_string()),
        };
    }
}
