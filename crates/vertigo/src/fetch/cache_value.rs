use std::rc::Rc;

use crate::{
    Computed, Context, DropResource, Value, driver_module::api::api_timers,
    fetch::api_response::ApiResponse, struct_mut::ValueMut,
};

#[derive(Clone)]
pub struct CacheValue {
    value: Value<ApiResponse>,
    /// What a computed or a subscription reads, see `get`. Its `when_connect` forgets the
    /// response when the bearer token changes, while the cache is observed.
    value_read: Computed<ApiResponse>,
}

/// Forgets the cached response whenever the bearer token changes, so that it's fetched again
/// with the new one.
fn revalidate_on_new_token(
    bearer_auth: &Computed<Option<String>>,
    last_token: &Rc<ValueMut<Option<Option<String>>>>,
    value_write: &Value<ApiResponse>,
) -> DropResource {
    let last_token = last_token.clone();
    let value_write = value_write.clone();

    let drop = bearer_auth.clone().subscribe(move |new_token| {
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

impl CacheValue {
    pub fn new(init_value: ApiResponse, bearer_auth: Computed<Option<String>>) -> CacheValue {
        let value = Value::new(init_value);

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

        let value_read = value.to_computed().when_connect({
            let value = value.clone();
            move || revalidate_on_new_token(&bearer_auth, &last_token, &value)
        });

        CacheValue { value, value_read }
    }

    /// The value. A read in a transaction gives what the cell holds now, as with a [`Value`] -
    /// also when the transaction has just set it. A computed or a subscription gets it as from
    /// a [`Computed`], brought up to date by the wave.
    pub fn get(&self, context: &Context) -> ApiResponse {
        if context.is_tracking() {
            // Through `value_read`: reading it keeps the token watched, and puts the reader two
            // computeds below the value, as deep as a `Router`'s render. The wave reaches nodes
            // level by level, so a write made first in a transaction - the address, when
            // leaving the page - then reaches that render before the readers of this cache
            // (see `crate::transaction`). Giving the cell's value here instead would make the
            // reader depend on a node the graph doesn't list as its parent.
            self.value_read.get(context)
        } else {
            // Straight from the cell: `value_read` read in an open transaction still has the
            // value from before its writes. A read outside of the graph's callbacks records no
            // parent, so nothing depends on it.
            self.value.get(context)
        }
    }

    pub fn set(&self, value: ApiResponse) {
        self.value.set(value);
    }
}
