use crate::{
    computed::{get_dependencies, Value},
    dev::command::{LocationSetMode, LocationTarget},
    driver_module::api::api_location,
    Computed, DomNode, EmbedDom, Reactive, ToComputed,
};

/// Router based on path or hash part of current location.
///
/// Note: If you want your app to support dynamic mount point,
/// you should use method [Driver::route_to_public](crate::Driver::route_to_public)
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
    location_target: LocationTarget,
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
        let location_target = match use_history_api {
            false => LocationTarget::Hash,
            true => LocationTarget::History,
        };

        let init_value = T::from(api_location().get_location(location_target));

        let route = Value::with_connect(init_value, move |value| {
            let value = value.clone();
            let callback = move |url: String| {
                value.set(T::from(url));
            };

            api_location().on_change(location_target, callback)
        });

        Self {
            location_target,
            route,
        }
    }

    pub fn set(&self, route: T) {
        api_location().push_location(
            self.location_target,
            LocationSetMode::Push,
            &route.to_string(),
        );
    }

    fn change(&self, change_fn: impl FnOnce(&mut T)) {
        get_dependencies().transaction(|ctx| {
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
