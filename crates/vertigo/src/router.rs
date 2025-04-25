use crate::{computed::Value, get_driver, Computed, DomNode, EmbedDom, Reactive, ToComputed};

/// Router based on path or hash part of current location.
///
/// Not: If you want your app to support dynamic mount point, you use method [Driver::route_to_public]
/// which will always prefix your route with mount point.
///
/// ```rust
/// use vertigo::{dom, DomNode, get_driver, router::Router};
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
/// impl std::fmt::Display for Route {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         let str = match self {
///             Self::Page1 => "/",
///             Self::Page2 => "/page2",
///             Self::NotFound => "/404",
///         };
///         f.write_str(&get_driver().route_to_public(str))
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

    fn change(&self, change_fn: impl FnOnce(&mut T)) {
        get_driver().inner.dependencies.transaction(|ctx| {
            let mut value = self.get(ctx);
            change_fn(&mut value);
            self.set(value);
        });
    }
}

impl<T: Clone + PartialEq + ToString + From<String>> Reactive<T> for Router<T> {
    fn set(&self, value: T) {
        Router::set(self, value)
    }

    fn get(&self, context: &crate::Context) -> T {
        self.route.get(context)
    }

    fn change(&self, change_fn: impl FnOnce(&mut T)) {
        Router::change(self, change_fn)
    }
}

impl<T: Clone + PartialEq + ToString + From<String>> ToComputed<T> for Router<T> {
    fn to_computed(&self) -> Computed<T> {
        self.route.to_computed()
    }
}

impl<T: Clone + PartialEq + ToString + From<String>> EmbedDom for Router<T> {
    fn embed(self) -> DomNode {
        self.route.embed()
    }
}

impl<T: Clone + PartialEq + ToString + From<String>> Default for Router<T> {
    fn default() -> Self {
        Router::new(true)
    }
}
