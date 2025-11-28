use std::rc::Rc;

use vertigo_macro::store;

use crate::{
    driver_module::api::api_browser_command, struct_mut::ValueMut, SsrFetchCache, SsrFetchRequest,
    SsrFetchResponse,
};

#[store]
pub fn api_fetch_cache() -> Rc<FetchCache> {
    Rc::new(FetchCache {
        cache: ValueMut::new(Rc::new(SsrFetchCache::empty())),
    })
}

pub struct FetchCache {
    cache: ValueMut<Rc<SsrFetchCache>>,
}

impl FetchCache {
    pub fn init_cache(&self) {
        let cache = api_browser_command().fetch_cache_get();
        self.cache.set(Rc::new(cache));
    }

    pub fn get_response(&self, request: &SsrFetchRequest) -> Option<SsrFetchResponse> {
        let cache = self.cache.get();
        cache.get(request).cloned()
    }
}
