use crate::{
    Computed, DropResource,
    computed::{Value, ValueSynchronize, context::Context},
    driver_module::api::api_timers,
    fetch::api_response::ApiResponse,
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

        let value_read: Computed<ApiResponse<T>> = {
            let value_write = value_write.clone();
            let bearer_auth = bearer_auth;

            value_write.to_computed().when_connect({
                let bearer_auth = bearer_auth.clone();

                move || {
                    let value_write = value_write.clone();

                    let revalidate_trigger = bearer_auth.clone();

                    let drop = revalidate_trigger.subscribe(move |_new_token| {
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

    pub fn synchronize<R: ValueSynchronize<std::rc::Rc<T>> + Clone + 'static>(
        &self,
    ) -> (R, DropResource)
    where
        T: Default + Clone,
    {
        use crate::{Resource, transaction};
        use std::rc::Rc;
        use vertigo_macro::bind;

        fn normalize<T: Default>(val: ApiResponse<T>) -> Rc<T> {
            match val {
                ApiResponse::Uninitialized => Rc::new(T::default()),
                ApiResponse::Data { value, expiry: _ } => match value {
                    Resource::Ready(data) => data,
                    Resource::Loading => Rc::new(T::default()),
                    Resource::Error(_) => Rc::new(T::default()),
                },
            }
        }

        let init_val = transaction(|ctx| normalize(self.value_write.get(ctx)));
        let target = R::new(init_val);

        let drop = self.value_write.add_event(bind!(target, |val| {
            target.set(normalize(val));
        }));

        (target, drop)
    }
}
