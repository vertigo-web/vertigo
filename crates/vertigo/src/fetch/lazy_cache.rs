use std::fmt::Debug;
use std::rc::Rc;

use crate::computed::context::Context;
use crate::{get_driver, Computed, transaction};
use crate::{
    Instant, Resource,
    computed::Value, struct_mut::ValueMut,
};

use super::request_builder::{RequestBuilder, RequestBody};

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

enum ApiResponse<T> {
    Notinit,
    Data {
        value: Resource<Rc<T>>,
        expiry: Option<Instant>,
    }
}

impl<T> ApiResponse<T> {
    pub fn new(value: Resource<Rc<T>>, expiry: Option<Instant>) -> Self {
        Self::Data {
            value,
            expiry
        }
    }

    pub fn new_loading() -> Self {
        ApiResponse::Data { value: Resource::Loading, expiry: None }
    }

    pub fn get_value(&self) -> Resource<Rc<T>> {
        match self {
            Self::Notinit => Resource::Loading,
            Self::Data { value, expiry: _ } => value.clone()
        }
    }

    pub fn needs_update(&self) -> bool {
        match self {
            ApiResponse::Notinit => true,
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
            ApiResponse::Notinit => ApiResponse::Notinit,
            ApiResponse::Data { value, expiry } => {
                ApiResponse::Data {
                    value: value.clone(),
                    expiry: expiry.clone(),
                }        
            }
        }
    }
}

/// A structure similar to Value but supports Loading/Error states and automatic refresh
/// after defined amount of time.
///
/// ```rust
/// use vertigo::{Computed, LazyCache, RequestBuilder, AutoJsJson, Resource};
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
/// See ["todo" example](../src/vertigo_demo/app/todo/mod.rs.html) in vertigo-demo package for more.
pub struct LazyCache<T: 'static> {
    id: u64,
    value: Value<ApiResponse<T>>,
    queued: Rc<ValueMut<bool>>,
    request: Rc<RequestBuilder>,
    map_response: Rc<dyn Fn(u32, RequestBody) -> Option<Resource<T>>>,
}

impl<T: 'static> Debug for LazyCache<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct ("LazyCache")
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
        map_response: impl Fn(u32, RequestBody) -> Option<Resource<T>> + 'static
    ) -> Self {
        Self {
            id: get_unique_id(),
            value: Value::new(ApiResponse::Notinit),
            queued: Rc::new(ValueMut::new(false)),
            request: Rc::new(request),
            map_response: Rc::new(map_response),
        }
    }

    pub fn get(&self, context: &Context) -> Resource<Rc<T>> {
        let api_response = self.value.get(context);

        //TODO - Add casch handling
        // let map_response = self.map_response.clone();

        // // if get_driver().is_cache_avaible() {
        // //     let fut = (self.loader)();
        // //     //pool the future only once
        // // }

        if !self.queued.get() && api_response.needs_update() {
            self.force_update_spawn(true);
        }

        api_response.get_value()
    }

    pub fn force_update(&self, with_loading: bool) {
        if !self.queued.get() {
            self.force_update_spawn(with_loading);
        }
    }

    fn force_update_spawn(&self, with_loading: bool) {
        get_driver().spawn({
            let queued = self.queued.clone();
            let value = self.value.clone();
            let request = self.request.clone();
            let map_response = self.map_response.clone();

            async move {
                if queued.get() {
                    return;
                }

                queued.set(true);   //set lock

                let api_response = transaction(|context| {
                    value.get(context)
                });
                
                if api_response.needs_update() {
                    //update value

                    if with_loading {
                        value.set(ApiResponse::new_loading());
                    }

                    let ttl = request.get_ttl();
                    let map_response = &(*map_response);
                    let new_value = request.call().await.into(map_response);

                    let expiry = ttl.map(|ttl| get_driver().now().add_duration(ttl));
                    
                    let new_value = new_value.map(Rc::new);
                    value.set(ApiResponse::new(new_value, expiry));
                }

                queued.set(false);
            }
        });
    }

    pub fn to_computed(&self) -> Computed<Resource<Rc<T>>> {
        Computed::from({
            let state = self.clone();
            move |context| {
                state.get(context)
            }
        })
    }
}

impl<T> PartialEq for LazyCache<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
