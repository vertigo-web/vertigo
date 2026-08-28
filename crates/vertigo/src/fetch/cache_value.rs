use std::rc::Rc;

use crate::{
    Computed, Context, DropResource, Value, driver_module::api::api_timers,
    fetch::api_response::ApiResponse, struct_mut::ValueMut,
};

pub struct CacheValue<T: PartialEq + 'static> {
    value_write: Value<ApiResponse<T>>,
    value_read: Computed<ApiResponse<T>>,
}

impl<T: PartialEq> Clone for CacheValue<T> {
    fn clone(&self) -> Self {
        CacheValue {
            value_write: self.value_write.clone(),
            value_read: self.value_read.clone(),
        }
    }
}

impl<T: PartialEq + 'static> CacheValue<T> {
    pub fn new(init_value: ApiResponse<T>, bearer_auth: Computed<Option<String>>) -> CacheValue<T> {
        let value_write = Value::new(init_value);

        // The token this cache was last revalidated against.
        //
        // `subscribe` runs its callback once on subscription, before anything has changed, and
        // discarding the value there would throw away exactly the thing that arrived with the
        // page: a response the server had already fetched into `data-fetch-cache`. Every
        // server-rendered `LazyCache` would then re-request on hydration, whether or not the
        // application uses bearer auth at all.
        //
        // So the token is remembered rather than the first call being counted. It lives out
        // here, not inside `when_connect`, because that closure runs again on every reconnect:
        // a token that changed while this value was disconnected still has to be noticed.
        let last_token: Rc<ValueMut<Option<Option<String>>>> = Rc::new(ValueMut::new(None));

        let value_read: Computed<ApiResponse<T>> = {
            let value_write = value_write.clone();
            let bearer_auth = bearer_auth;

            value_write.to_computed().when_connect({
                let bearer_auth = bearer_auth.clone();

                move || {
                    let value_write = value_write.clone();
                    let last_token = last_token.clone();

                    let revalidate_trigger = bearer_auth.clone();

                    let drop = revalidate_trigger.subscribe(move |new_token| {
                        let changed = last_token.change(|last| match last {
                            Some(previous) if *previous == new_token => false,
                            _ => {
                                let first = last.is_none();
                                *last = Some(new_token.clone());
                                !first
                            }
                        });

                        if !changed {
                            return;
                        }

                        let value_write = value_write.clone();

                        api_timers().set_timeout_and_detach(0, move || {
                            value_write.set(ApiResponse::Uninitialized);
                        });
                    });

                    DropResource::new(move || {
                        drop.off();
                    })
                }
            })
        };

        CacheValue {
            value_write,
            value_read,
        }
    }

    pub fn get(&self, context: &Context) -> ApiResponse<T> {
        self.value_read.get(context)
    }

    pub fn set(&self, value: ApiResponse<T>) {
        self.value_write.set(value);
    }
}
