use std::fmt::Debug;
use std::rc::Rc;

use crate::{
    Computed, DomNode, Instant, JsJsonDeserialize, RequestResponse, Resource, ToComputed, computed::{Value, context::Context}, driver_module::api::api_fetch_cache, get_driver, struct_mut::ValueMut, transaction
};

use super::request_builder::{RequestBody, RequestBuilder};

type MapResponse<T> = Option<Result<T, String>>;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(PartialEq)]
enum ApiResponse<T> {
    Uninitialized,
    Data {
        value: Resource<Rc<T>>,
        expiry: Option<Instant>,
    },
}

impl<T> ApiResponse<T> {
    pub fn new(value: Resource<Rc<T>>, expiry: Option<Instant>) -> Self {
        Self::Data { value, expiry }
    }

    pub fn new_loading() -> Self {
        ApiResponse::Data {
            value: Resource::Loading,
            expiry: None,
        }
    }

    pub fn get_value(&self) -> Resource<Rc<T>> {
        match self {
            Self::Uninitialized => Resource::Loading,
            Self::Data { value, expiry: _ } => value.clone(),
        }
    }

    pub fn needs_update(&self) -> bool {
        match self {
            ApiResponse::Uninitialized => true,
            ApiResponse::Data { value: _, expiry } => {
                let Some(expiry) = expiry else {
                    return false;
                };

                expiry.is_expire()
            }
        }
    }
}

impl<T> Clone for ApiResponse<T> {
    fn clone(&self) -> Self {
        match self {
            ApiResponse::Uninitialized => ApiResponse::Uninitialized,
            ApiResponse::Data { value, expiry } => ApiResponse::Data {
                value: value.clone(),
                expiry: expiry.clone(),
            },
        }
    }
}

/// A structure similar to [Value] but supports Loading/Error states and automatic refresh
/// after defined amount of time.
///
/// ```rust
/// use vertigo::{LazyCache, RequestBuilder, AutoJsJson};
///
/// #[derive(AutoJsJson, PartialEq, Clone)]
/// pub struct Model {
///     id: i32,
///     name: String,
/// }
///
/// pub struct TodoState {
///     posts: LazyCache<Vec<Model>>,
/// }
///
/// impl TodoState {
///     pub fn new() -> Self {
///         let posts = RequestBuilder::get("https://some.api/posts")
///             .ttl_seconds(300)
///             .lazy_cache(|status, body| {
///                 if status == 200 {
///                     Some(body.into::<Vec<Model>>())
///                 } else {
///                     None
///                 }
///             });
///
///         TodoState {
///             posts
///         }
///     }
/// }
/// ```
///
/// See ["todo" example](../src/vertigo_demo/app/todo/state.rs.html) in vertigo-demo package for more.
pub struct LazyCache<T: 'static> {
    id: u64,
    value: Value<ApiResponse<T>>,
    queued: Rc<ValueMut<bool>>,
    request: Rc<RequestBuilder>,
    map_response: Rc<dyn Fn(u32, RequestBody) -> MapResponse<T>>,
}

impl<T: 'static> Debug for LazyCache<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyCache")
            .field("queued", &self.queued)
            .finish()
    }
}

impl<T> Clone for LazyCache<T> {
    fn clone(&self) -> Self {
        LazyCache {
            id: self.id,
            value: self.value.clone(),
            queued: self.queued.clone(),
            request: self.request.clone(),
            map_response: self.map_response.clone(),
        }
    }
}

impl<T> LazyCache<T> {
    pub fn new(
        request: RequestBuilder,
        map_response: impl Fn(u32, RequestBody) -> MapResponse<T> + 'static,
    ) -> Self {

        let map_response = Rc::new(map_response);

        let init_value: ApiResponse<T> = {
            let request = request.clone();
            let map_response = map_response.clone();

            let ttl = request.get_ttl();

            let ssr_request = request.to_request();
            if let Some(response) = api_fetch_cache().get_response(&ssr_request) {

                
                let response = RequestResponse::new(ssr_request, response);

                let new_value = response.into(map_response.as_ref());

                let new_value = match new_value {
                    Ok(value) => Resource::Ready(Rc::new(value)),
                    Err(message) => Resource::Error(message),
                };

                let expiry = ttl.map(|ttl| get_driver().now().add_duration(ttl));

                ApiResponse::new(new_value, expiry)
            } else {
                ApiResponse::Uninitialized
            }
        };

        Self {
            id: get_unique_id(),
            value: Value::new(init_value),
            queued: Rc::new(ValueMut::new(false)),
            request: Rc::new(request),
            map_response,
        }
    }
}

impl<T: PartialEq> LazyCache<T> {
    /// Get value (update if needed)
    pub fn get(&self, context: &Context) -> Resource<Rc<T>> {
        let api_response = self.value.get(context);

        if !self.queued.get() && api_response.needs_update() {
            self.update(false, false);
        }

        api_response.get_value()
    }

    /// Delete value so it will refresh on next access
    pub fn forget(&self) {
        self.value.set(ApiResponse::Uninitialized);
    }

    /// Force refresh the value now
    pub fn force_update(&self, with_loading: bool) {
        self.update(with_loading, true)
    }

    /// Update the value if expired
    pub fn update(&self, with_loading: bool, force: bool) {
        if self.queued.get() {
            return;
        }

        self.queued.set(true); //set lock

        let self_clone = self.clone();

        get_driver().spawn(async move {
            if !self_clone.queued.get() {
                log::error!("force_update_spawn: queued.get() in spawn -> expected false");
                return;
            }

            let api_response = transaction(|context| self_clone.value.get(context));

            if force || api_response.needs_update() {
                if with_loading {
                    self_clone.value.set(ApiResponse::new_loading());
                }

                let new_value = self_clone
                    .request
                    .as_ref()
                    .clone()
                    .call()
                    .await
                    .into(self_clone.map_response.as_ref());

                let new_value = match new_value {
                    Ok(value) => Resource::Ready(Rc::new(value)),
                    Err(message) => Resource::Error(message),
                };

                let expiry = self_clone
                    .request
                    .get_ttl()
                    .map(|ttl| get_driver().now().add_duration(ttl));

                self_clone.value.set(ApiResponse::new(new_value, expiry));
            }

            self_clone.queued.set(false);
        });
    }
}

impl<T: Clone + PartialEq> ToComputed<Resource<Rc<T>>> for LazyCache<T> {
    fn to_computed(&self) -> Computed<Resource<Rc<T>>> {
        Computed::from({
            let state = self.clone();
            move |context| state.get(context)
        })
    }
}

impl<T: Clone + PartialEq> LazyCache<T> {
    pub fn to_computed(&self) -> Computed<Resource<Rc<T>>> {
        <Self as ToComputed<Resource<Rc<T>>>>::to_computed(self)
    }
}

impl<T> PartialEq for LazyCache<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: PartialEq + Clone> LazyCache<T> {
    pub fn render(&self, render: impl Fn(Rc<T>) -> DomNode + 'static) -> DomNode {
        self.to_computed().render_value(move |value| match value {
            Resource::Ready(value) => render(value),
            Resource::Loading => {
                use crate as vertigo;

                vertigo::dom! {
                    <div>
                        "Loading ..."
                    </div>
                }
            }
            Resource::Error(error) => {
                use crate as vertigo;

                vertigo::dom! {
                    <div>
                        "error = "
                        {error}
                    </div>
                }
            }
        })
    }
}

impl<T: JsJsonDeserialize> LazyCache<T> {
    /// Helper to easily create a lazy cache of `Vec<T>` deserialized from provided URL base and route
    ///
    /// ```rust
    /// use vertigo::{LazyCache, AutoJsJson};
    ///
    /// #[derive(AutoJsJson, PartialEq, Clone)]
    /// pub struct Model {
    ///     id: i32,
    ///     name: String,
    /// }
    ///
    /// let posts = LazyCache::<Vec<Model>>::new_resource("https://some.api", "/posts", 60);
    /// ```
    pub fn new_resource(api: &str, path: &str, ttl: u64) -> Self {
        let url = [api, path].concat();

        LazyCache::new(
            get_driver().request_get(url).ttl_seconds(ttl),
            |status, body| {
                if status == 200 {
                    Some(body.into::<T>())
                } else {
                    None
                }
            },
        )
    }
}
