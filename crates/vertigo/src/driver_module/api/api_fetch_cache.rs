use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    dev::{SsrFetchCache, SsrFetchRequest, SsrFetchResponse},
    struct_mut::ValueMut,
};

use super::api_browser_command;

#[store]
pub fn api_fetch_cache() -> Rc<FetchCache> {
    Rc::new(FetchCache {
        cache: ValueMut::new(None),
    })
}

/// The responses the server already fetched while rendering this page, decoded on first ask.
///
/// Decoding is deferred rather than done at startup so that an application with no
/// [`LazyCache`](crate::LazyCache) never mentions it. `get_response` is the only caller, and
/// the only call to `get_response` is the one in `LazyCache::new`, so in an application that
/// fetches nothing the linker can drop this whole file and the `SsrFetchCache` decoder behind
/// it - a tenth of the wasm of a small application. It used to be pulled in unconditionally
/// from `start_app`.
pub struct FetchCache {
    cache: ValueMut<Option<Rc<SsrFetchCache>>>,
}

impl FetchCache {
    pub fn get_response(&self, request: &SsrFetchRequest) -> Option<SsrFetchResponse> {
        let cache = match self.cache.get() {
            Some(cache) => cache,
            None => {
                // Two things make deferring this safe. The cache travels in a
                // `data-fetch-cache` attribute on `v-metadata`, which the js side detaches
                // from the document at startup but keeps a reference to, so it reads the same
                // now as it would have at boot. And the memoized value cannot outlive the page
                // it came from: the store behind `api_fetch_cache` is a thread local, the
                // browser gets a fresh module per page load, and the server builds a fresh
                // `WasmInstance` per request.
                let cache = Rc::new(api_browser_command().fetch_cache_get());
                self.cache.set(Some(cache.clone()));
                cache
            }
        };

        cache.get(request).cloned()
    }
}
