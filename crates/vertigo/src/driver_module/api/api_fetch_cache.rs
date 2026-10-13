use std::{cell::Cell, rc::Rc};
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

thread_local! {
    static CLOSED: Cell<bool> = const { Cell::new(false) };
}

/// Stops [`FetchCache`] from handing out responses. `mount` calls it once the first render is done.
///
/// The responses are the data as it was when the server rendered the page, which is what the
/// first render needs to build the same page again. A cache created later - f. ex. by a route
/// rendered anew after navigating away and back - would start with data that may have changed
/// since, the application's own changes included, so it fetches instead, as it would on a page
/// the server didn't render.
///
/// A flag of its own rather than a method of `FetchCache`, so that `mount` calling it doesn't
/// bring `FetchCache` into an application that never uses it.
pub fn close_fetch_cache() {
    CLOSED.with(|closed| closed.set(true));
}

/// The responses the server already fetched while rendering this page, decoded on first ask.
///
/// Decoding is deferred rather than done at startup so that an application with no
/// [`LazyCache`](crate::LazyCache) or [`LazyListCache`](crate::LazyListCache) never mentions it.
/// `get_response` is the only caller, and the only calls to `get_response` are the ones in
/// `LazyCache::new` and `LazyListCache::new`, so in an application that fetches nothing
/// the linker can drop `FetchCache` and the `SsrFetchCache` decoder behind it - a tenth of
/// the wasm of a small application. It used to be pulled in unconditionally from `start_app`.
pub struct FetchCache {
    cache: ValueMut<Option<Rc<SsrFetchCache>>>,
}

impl FetchCache {
    /// The response the server got for `request`, until the first render is done
    /// (see [`close_fetch_cache`]).
    pub fn get_response(&self, request: &SsrFetchRequest) -> Option<SsrFetchResponse> {
        if CLOSED.with(Cell::get) {
            return None;
        }

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

/// Pretends the server fetched `url` while rendering the page, as it embeds the response in
/// `data-fetch-cache`.
#[cfg(test)]
pub fn ssr_fetched(url: &str, status: u32, body: crate::JsJson) {
    use crate::{
        JsJsonSerialize, RequestBuilder, dev::SsrFetchResponseContent,
        external_api::safe_wrappers::TEST_FETCH_CACHE,
    };
    use std::collections::BTreeMap;

    let request = RequestBuilder::get(url).to_request(None);
    let response = SsrFetchResponse::Ok {
        status,
        response: SsrFetchResponseContent::Json(body),
    };
    let cache = SsrFetchCache::new(&BTreeMap::from([(request, response)]));

    TEST_FETCH_CACHE.with(|stored| {
        *stored.borrow_mut() = Some(cache.to_json().convert_to_string());
    });
}
