use std::fmt::Debug;
use std::rc::Rc;

use crate::{
    Computed, DomNode, DropResource, JsJsonDeserialize, RequestResponse, Resource, ToComputed,
    computed::{ValueSynchronize, context::Context, struct_mut::ValueMut},
    driver_module::api::{api_fetch, api_fetch_cache},
    fetch::{api_response::ApiResponse, cache_value::CacheValue},
    get_driver, transaction,
};

use super::request_builder::{RequestBody, RequestBuilder};

type MapResponse<T> = Option<Result<T, String>>;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
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
pub struct LazyCache<T: PartialEq + 'static> {
    id: u64,
    value: CacheValue<T>,
    queued: Rc<ValueMut<bool>>,
    request: Rc<RequestBuilder>,
    map_response: Rc<dyn Fn(u32, RequestBody) -> MapResponse<T>>,
}

impl<T: PartialEq + 'static> Debug for LazyCache<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyCache")
            .field("queued", &self.queued)
            .finish()
    }
}

impl<T: PartialEq> Clone for LazyCache<T> {
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

impl<T: PartialEq> LazyCache<T> {
    pub fn new(
        request: RequestBuilder,
        map_response: impl Fn(u32, RequestBody) -> MapResponse<T> + 'static,
    ) -> Self {
        let map_response = Rc::new(map_response);

        let init_value: ApiResponse<T> = {
            let request = request.clone();
            let map_response = map_response.clone();

            let ttl = request.get_ttl();

            let ssr_request = request.to_request(None);
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
            value: CacheValue::new(init_value, request.get_bearer_auth()),
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

                let request = transaction(|context| {
                    self_clone
                        .request
                        .as_ref()
                        .clone()
                        .to_request_context(context)
                });

                let result = api_fetch().fetch(request.clone()).await;
                let new_value =
                    RequestResponse::new(request, result).into(self_clone.map_response.as_ref());

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

    pub fn synchronize<R: ValueSynchronize<std::rc::Rc<T>> + Clone + 'static>(
        &self,
    ) -> (R, DropResource)
    where
        T: Default + Clone,
    {
        self.value.synchronize()
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

impl<T: PartialEq> PartialEq for LazyCache<T> {
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

impl<T: PartialEq + JsJsonDeserialize> LazyCache<T> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::{SsrFetchResponse, SsrFetchResponseContent};
    use crate::driver_module::api::api_timers;
    use crate::{JsJson, Value};

    #[tokio::test]
    async fn test_lazy_cache_initial_state() {
        let cache = RequestBuilder::get("https://test.com/api").lazy_cache(|status, body| {
            if status == 200 {
                Some(body.into::<String>())
            } else {
                None
            }
        });

        transaction(|context| {
            let result = cache.get(context);
            assert_eq!(result, Resource::Loading);
        });
    }

    #[tokio::test]
    async fn test_lazy_cache_to_computed() {
        let cache = RequestBuilder::get("https://test.com/api").lazy_cache(|status, body| {
            if status == 200 {
                Some(body.into::<String>())
            } else {
                None
            }
        });

        let computed = cache.to_computed();

        transaction(|context| {
            let result = computed.get(context);
            assert_eq!(result, Resource::Loading);
        });
    }

    #[tokio::test]
    async fn test_lazy_cache_token_revalidation() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                // Mock timers to execute 0ms timeouts in next tick
                api_timers().set_mock_handler(|duration, callback_id, _kind| {
                    if duration == 0 {
                        tokio::task::spawn_local(async move {
                            api_timers().callback_timeout(callback_id);
                        });
                    }
                });

                let token = Value::new(Some("token1".to_string()));

                let cache = RequestBuilder::get("https://test.com/api")
                    .bearer_auth(token.to_computed())
                    .lazy_cache(|status, body| {
                        if status == 200 {
                            Some(body.into::<String>())
                        } else {
                            None
                        }
                    });

                let api = api_fetch();
                let call_count = Rc::new(crate::dev::ValueMut::new(0));

                {
                    let call_count = call_count.clone();
                    api.set_mock_handler(move |request| {
                        call_count.change(|val| *val += 1);
                        let auth = request
                            .headers
                            .get("Authorization")
                            .cloned()
                            .unwrap_or_default();
                        SsrFetchResponse::Ok {
                            status: 200,
                            response: SsrFetchResponseContent::Json(JsJson::String(format!(
                                "response for {}",
                                auth
                            ))),
                        }
                    });
                }

                // 1. Initial access and keep observed
                let _drop = cache.to_computed().subscribe(|_| {});

                transaction(|context| {
                    let res = cache.get(context);
                    assert_eq!(res, Resource::Loading);
                });

                // Small sleep to let spawned tasks run
                tokio::task::yield_now().await;

                // 2. Check data
                transaction(|context| {
                    let res = cache.get(context);
                    if let Resource::Ready(val) = res {
                        assert_eq!(val.as_str(), "response for Bearer token1");
                    } else {
                        panic!("Expected Ready, got {:?}", res);
                    }
                });
                // It fetches twice: once in new(), and once when bearer_auth.subscribe triggers initially (which sets Uninitialized)
                assert_eq!(call_count.get(), 2);

                // 3. Change token
                transaction(|_| {
                    token.set(Some("token2".to_string()));
                });

                // Wait for revalidation
                tokio::task::yield_now().await;
                tokio::task::yield_now().await;

                // 4. Check data again
                transaction(|context| {
                    let res = cache.get(context);
                    if let Resource::Ready(val) = res {
                        assert_eq!(val.as_str(), "response for Bearer token2");
                    } else {
                        panic!("Expected Ready after token change, got {:?}", res);
                    }
                });
                assert_eq!(call_count.get(), 3);
            })
            .await;
    }

    #[tokio::test]
    async fn test_lazy_cache_set_same_computed_token() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                // Mock timers to execute 0ms timeouts in next tick
                api_timers().set_mock_handler(|duration, callback_id, _kind| {
                    if duration == 0 {
                        tokio::task::spawn_local(async move {
                            api_timers().callback_timeout(callback_id);
                        });
                    }
                });

                let token_val = Value::new(Some("token1".to_string()));
                let token_computed = token_val.to_computed();

                let rb =
                    RequestBuilder::get("https://test.com/api").bearer_auth(token_computed.clone());

                let call_count = Rc::new(crate::dev::ValueMut::new(0));
                {
                    let call_count = call_count.clone();
                    api_fetch().set_mock_handler(move |request| {
                        call_count.change(|val| *val += 1);
                        let auth = request
                            .headers
                            .get("Authorization")
                            .cloned()
                            .unwrap_or_default();
                        SsrFetchResponse::Ok {
                            status: 200,
                            response: SsrFetchResponseContent::Json(JsJson::String(format!(
                                "resp:{}",
                                auth
                            ))),
                        }
                    });
                }

                let cache = rb.clone().lazy_cache(|status, body| {
                    if status == 200 {
                        Some(body.into::<String>())
                    } else {
                        None
                    }
                });

                let _drop = cache.to_computed().subscribe(|_| {});

                tokio::task::yield_now().await;
                tokio::task::yield_now().await;

                transaction(|context| {
                    let res = cache.get(context);
                    if let Resource::Ready(val) = res {
                        assert_eq!(val.as_str(), "resp:Bearer token1");
                    } else {
                        panic!("Expected Ready, got {:?}", res);
                    }
                });
                assert_eq!(call_count.get(), 2);

                // Setting SAME token computed again on the request builder
                let _ = rb.bearer_auth(token_computed.clone());

                tokio::task::yield_now().await;
                tokio::task::yield_now().await;

                // Should NOT have fetched again yet because token content didn't change and Computed instance is the same
                assert_eq!(call_count.get(), 2);

                // Change content of the same Computed
                transaction(|_| {
                    token_val.set(Some("token2".to_string()));
                });

                tokio::task::yield_now().await;
                tokio::task::yield_now().await;

                transaction(|context| {
                    let res = cache.get(context);
                    if let Resource::Ready(val) = res {
                        assert_eq!(val.as_str(), "resp:Bearer token2");
                    } else {
                        panic!("Expected Ready after token change, got {:?}", res);
                    }
                });
                assert_eq!(call_count.get(), 3);
            })
            .await;
    }
}
