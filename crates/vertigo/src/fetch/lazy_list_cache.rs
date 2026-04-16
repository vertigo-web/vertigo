use std::{collections::HashSet, rc::Rc};

use crate::{
    Computed, DomNode, JsJsonDeserialize, RequestResponse, Resource, Value,
    computed::{
        context::Context,
        struct_mut::{HashMapMut, ValueMut},
    },
    driver_module::api::api_fetch,
    fetch::request_builder::{RequestBody, RequestBuilder},
    get_driver,
    render::collection::CollectionKey,
    transaction,
};

pub type MapResponse<V> = Option<Result<V, String>>;
pub type ItemRequestCallback<K> = Rc<dyn Fn(&K) -> RequestBuilder>;
pub type MapResponseCallback<V> = Rc<dyn Fn(u32, RequestBody) -> MapResponse<V>>;
pub type GranularItem<T> = Computed<Option<T>>;
pub type GranularList<T> = Resource<Rc<Vec<GranularItem<T>>>>;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Per-item storage holding the server-provided original and an optional optimistic override.
pub struct ListItem<V: Clone + PartialEq + 'static> {
    pub original: Value<V>,
    /// `None` = no active override;
    /// `Some(None)` = optimistically removed;
    /// `Some(Some(v))` = optimistic override in effect.
    pub override_val: Value<Option<Option<V>>>,
}

impl<V: Clone + PartialEq + 'static> ListItem<V> {
    fn new(value: V) -> Self {
        ListItem {
            original: Value::new(value),
            override_val: Value::new(None),
        }
    }
}

#[derive(Clone, PartialEq)]
enum ListCacheState {
    Uninitialized,
    Loading,
    Ready,
    Error(String),
}

/// A lazy cache for lists that stores items in an ordered dict `T::Key -> ListItem<T::Value>`.
///
/// Each item has a reactive `original` (from the server) and an optional `override_val`
/// (from [`optimistically_set_item`](LazyListCache::optimistically_set_item)).
/// [`get_by_key`](LazyListCache::get_by_key) returns the override when active, otherwise the original.
///
/// `T` is a marker type implementing [`CollectionKey`].
///
/// See [Lazy List](https://github.com/vertigo-web/vertigo/blob/optimistic-update/demo/app/src/app/lazy_list/state.rs) example in the demo.
///
/// ## State transitions
///
/// ```text
/// UPDATE FLOW -- editing a server-confirmed item (key=2)
///
///  ┌────────────────────────────────┐
///  │ keys: ..2..                    │
///  │ orig=v0, override=None         │
///  │ placeholder: empty             │
///  └────────────────────────────────┘
///                  │
///                  │ optimistically_set_item
///                  ▼
///  ┌────────────────────────────────┐
///  │ keys: ..2..                    │
///  │ orig=v0, override=Some(v1)     │
///  │ placeholder: empty             │
///  └────────────────────────────────┘
///          │                   │
///    update_item            rollback
///     (success)              (failure)
///          │                   │
///          ▼                   ▼
///  ┌──────────────────┐  ┌──────────────────┐
///  │ keys: ..2..      │  │ keys: ..2..      │
///  │ orig=v1          │  │ orig=v0          │
///  │ override=None    │  │ override=None    │
///  │ placeholder: -   │  │ placeholder: -   │
///  └──────────────────┘  └──────────────────┘
///
///
/// CREATE FLOW -- new item, temp id=0, server assigns id=42
///
///  ┌────────────────────────────────┐
///  │ keys: 1 2 3                    │
///  │ placeholder: empty             │
///  └────────────────────────────────┘
///                  │
///                  │ optimistically_set_item
///                  ▼
///  ┌──────────────────────────────────────┐
///  │ keys: 1 2 3 0                        │
///  │ orig=draft, override=Some(draft)     │
///  │ placeholder: 0                       │
///  └──────────────────────────────────────┘
///          │                            │
///  update_item_with_old_key         rollback
///        (success)                  (failure)
///          │                            │
///          ▼                            ▼
///  ┌──────────────────────┐  ┌──────────────────┐
///  │ keys: 1 2 3 42       │  │ keys: 1 2 3      │
///  │ orig=saved           │  │ placeholder: -   │
///  │ override=None        │  └──────────────────┘
///  │ placeholder: -       │
///  └──────────────────────┘
/// ```
pub struct LazyListCache<T: CollectionKey + 'static> {
    id: u64,
    state: Value<ListCacheState>,
    /// Insertion-ordered list of currently-visible keys: keys from the last server response plus
    /// any keys that were inserted optimistically and have not been rolled back. Reactive, so
    /// consumers (e.g. [`granular`](LazyListCache::granular)) re-evaluate when items are added,
    /// removed, or re-keyed via mutation methods.
    keys: Value<Vec<T::Key>>,
    /// Per-item reactive data, keyed for O(1) lookup.
    items: Rc<HashMapMut<T::Key, ListItem<T::Value>>>,

    /// Request for the list
    list_request: Rc<RequestBuilder>,
    /// If request for the list is queued
    list_queued: Rc<ValueMut<bool>>,
    /// Map response for the list
    list_map_response: MapResponseCallback<Vec<T::Value>>,

    /// Request for the items
    item_request: Option<ItemRequestCallback<T::Key>>,
    /// If request for particular item is queued
    item_queued: Rc<HashMapMut<T::Key, ()>>,
    /// Map response for the items
    item_map_response: Option<MapResponseCallback<T::Value>>,

    /// Keys that exist only as optimistic placeholders (not yet confirmed by the server).
    /// [`rollback`](LazyListCache::rollback) fully removes these instead of merely clearing
    /// the override, so a failed create doesn't leave its draft visible in the list.
    optimistic_placeholders: Rc<HashMapMut<T::Key, ()>>,
}

impl<T: CollectionKey> std::fmt::Debug for LazyListCache<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyListCache")
            .field("id", &self.id)
            .finish()
    }
}

impl<T: CollectionKey> Clone for LazyListCache<T> {
    fn clone(&self) -> Self {
        LazyListCache {
            id: self.id,
            state: self.state.clone(),
            keys: self.keys.clone(),
            items: self.items.clone(),
            list_queued: self.list_queued.clone(),
            list_request: self.list_request.clone(),
            list_map_response: self.list_map_response.clone(),
            item_request: self.item_request.clone(),
            item_map_response: self.item_map_response.clone(),
            item_queued: self.item_queued.clone(),
            optimistic_placeholders: self.optimistic_placeholders.clone(),
        }
    }
}

impl<T: CollectionKey> PartialEq for LazyListCache<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: CollectionKey> LazyListCache<T> {
    pub fn new(
        request: RequestBuilder,
        map_response: impl Fn(u32, RequestBody) -> MapResponse<Vec<T::Value>> + 'static,
    ) -> Self {
        LazyListCache {
            id: get_unique_id(),
            state: Value::new(ListCacheState::Uninitialized),
            keys: Value::new(Vec::new()),
            items: Rc::new(HashMapMut::new()),
            list_queued: Rc::new(ValueMut::new(false)),
            list_request: Rc::new(request),
            list_map_response: Rc::new(map_response),
            item_request: None,
            item_map_response: None,
            item_queued: Rc::new(HashMapMut::new()),
            optimistic_placeholders: Rc::new(HashMapMut::new()),
        }
    }

    /// Configure per-item fetching. Returns `self` for chaining.
    ///
    /// `item_request` builds a `RequestBuilder` for a given key (e.g. `GET /items/{id}`).
    /// `item_map_response` parses the response into a single `T::Value`.
    pub fn with_item_fetch(
        self,
        item_request: impl Fn(&T::Key) -> RequestBuilder + 'static,
        item_map_response: impl Fn(u32, RequestBody) -> MapResponse<T::Value> + 'static,
    ) -> Self {
        LazyListCache {
            item_request: Some(Rc::new(item_request)),
            item_map_response: Some(Rc::new(item_map_response)),
            ..self
        }
    }

    /// Fetch a single item by key and update its `original` value in the cache.
    /// No-op if per-item fetching was not configured via [`with_item_fetch`](Self::with_item_fetch).
    /// Deduplicates in-flight requests for the same key.
    pub fn fetch_item(&self, key: T::Key) {
        let (req_fn, resp_fn) = match (&self.item_request, &self.item_map_response) {
            (Some(r), Some(m)) => (r.clone(), m.clone()),
            _ => return,
        };

        if self.item_queued.get_and_map(&key, |_| ()).is_some() {
            return;
        }
        self.item_queued.insert(key.clone(), ());

        let self_clone = self.clone();
        get_driver().spawn(async move {
            let request = transaction(|context| req_fn(&key).to_request_context(context));
            let result = api_fetch().fetch(request.clone()).await;
            let new_value = RequestResponse::new(request, result).into(resp_fn.as_ref());

            if let Ok(item) = new_value {
                let item_key = T::get_key(&item);
                if self_clone
                    .items
                    .must_change(&item_key, |list_item| list_item.original.set(item.clone()))
                    .is_none()
                {
                    // Item not yet in cache — insert it and append to key order.
                    self_clone
                        .items
                        .insert(item_key.clone(), ListItem::new(item));
                    self_clone.keys.change(|current_keys| {
                        if !current_keys.contains(&item_key) {
                            current_keys.push(item_key);
                        }
                    });
                }
            }

            self_clone.item_queued.retain(|k, _| *k != key);
        });
    }

    /// Get the full list of items in insertion order. Yields each item's `original` value —
    /// pending overrides from [`optimistically_set_item`](Self::optimistically_set_item) are not
    /// applied here, so an optimistic *edit* still reads as the server's last-known value. Use
    /// [`get_by_key`](Self::get_by_key) when you need the override-aware view of a single item.
    ///
    /// Optimistic *inserts* (placeholders for brand-new items) do appear in the list, since the
    /// placeholder's `original` is seeded with the optimistic value at insert time.
    pub fn get(&self, context: &Context) -> Resource<Rc<Vec<T::Value>>> {
        let state = self.state.get(context);

        if state == ListCacheState::Uninitialized && !self.list_queued.get() {
            self.update(false, false);
        }

        match state {
            ListCacheState::Uninitialized | ListCacheState::Loading => Resource::Loading,
            ListCacheState::Error(e) => Resource::Error(e),
            ListCacheState::Ready => {
                let keys = self.keys.get(context);
                let list = keys
                    .iter()
                    .filter_map(|k| self.items.get_and_map(k, |item| item.original.get(context)))
                    .collect::<Vec<_>>();
                Resource::Ready(Rc::new(list))
            }
        }
    }

    /// Get the list as individually reactive [`Computed`] items, each yielding `Option<T::Value>`.
    ///
    /// Unlike [`get`](Self::get), which returns the whole list as a single reactive unit, this
    /// method wraps every element in its own [`Computed`]. Subscribers re-evaluate only for the
    /// specific items that changed, making it efficient for large lists with sparse updates.
    ///
    /// When `filter` is `Some`, each `Computed` applies the predicate and returns `None` for
    /// items that do not pass, allowing the UI to hide or skip them without recomputing the rest.
    pub fn granular<F>(&self, ctx: &Context, filter: Option<F>) -> GranularList<T::Value>
    where
        F: Fn(&T::Value) -> bool + Clone + 'static,
    {
        let state = self.state.get(ctx);

        if state == ListCacheState::Uninitialized && !self.list_queued.get() {
            self.update(false, false);
        }

        match state {
            ListCacheState::Uninitialized | ListCacheState::Loading => Resource::Loading,
            ListCacheState::Error(e) => Resource::Error(e),
            ListCacheState::Ready => {
                let keys = self.keys.get(ctx);
                let items = self.items.clone();
                let list = keys
                    .into_iter()
                    .map(|k| {
                        let items = items.clone();
                        let filter = filter.clone();
                        Computed::from(move |ctx| {
                            let item = items.get_and_map(&k, |item| item.original.get(ctx));
                            if let Some(filter) = &filter {
                                item.filter(filter)
                            } else {
                                item
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                Resource::Ready(Rc::new(list))
            }
        }
    }

    /// Get one item by key. Returns the active override when set, otherwise the original.
    /// Returns `Resource::Loading` when the list has not yet loaded (unless an override is set).
    pub fn get_by_key(&self, context: &Context, key: &T::Key) -> Resource<Rc<T::Value>> {
        let state = self.state.get(context);

        if state == ListCacheState::Uninitialized && !self.list_queued.get() {
            self.update(false, false);
        }

        // Override wins regardless of fetch state.
        match self
            .items
            .get_and_map(key, |item| item.override_val.get(context))
        {
            Some(Some(Some(v))) => return Resource::Ready(Rc::new(v)),
            Some(Some(None)) => return Resource::Loading,
            _ => (),
        }

        match state {
            ListCacheState::Uninitialized | ListCacheState::Loading => Resource::Loading,
            ListCacheState::Error(e) => Resource::Error(e),
            ListCacheState::Ready => {
                match self
                    .items
                    .get_and_map(key, |item| item.original.get(context))
                {
                    Some(v) => Resource::Ready(Rc::new(v)),
                    None => Resource::Error(format!("key {key:?} not found")),
                }
            }
        }
    }

    /// Clear the cache so it will refetch on next access.
    pub fn forget(&self) {
        self.state.set(ListCacheState::Uninitialized);
    }

    /// Force a refetch now.
    pub fn force_update(&self, with_loading: bool) {
        self.update(with_loading, true);
    }

    /// Insert or update an optimistic override for one item. Does not modify the server-originated data.
    /// If the key is not yet in the dict (e.g. cache is still loading or this is a brand-new item),
    /// a placeholder entry is created and the key is appended to the ordered key list so the item
    /// becomes immediately visible to consumers iterating the list.
    pub fn optimistically_set_item(&self, item: T::Value) {
        let key = T::get_key(&item);
        let updated = self.items.must_change(&key, |list_item| {
            list_item.override_val.set(Some(Some(item.clone())))
        });
        if updated.is_none() {
            // Key not yet loaded from server — create a placeholder and register the key.
            let list_item = ListItem {
                original: Value::new(item.clone()),
                override_val: Value::new(Some(Some(item))),
            };
            self.items.insert(key.clone(), list_item);
            self.optimistic_placeholders.insert(key.clone(), ());
            self.keys.change(|current_keys| {
                if !current_keys.contains(&key) {
                    current_keys.push(key);
                }
            });
        }
    }

    /// Optimistically mark an item as removed. The entry stays in the cache but
    /// [`get_by_key`](Self::get_by_key) reports it as `Resource::Loading` until either
    /// [`remove_item`](Self::remove_item) confirms the deletion or [`rollback`](Self::rollback)
    /// reverts the override. No-op if the key is not in the cache.
    pub fn optimistically_remove_item(&self, key: &T::Key) {
        self.items
            .must_change(key, |list_item| list_item.override_val.set(Some(None)));
    }

    /// Provide a new item (f. ex. returned from a create or update request) and remove the override.
    /// Always ensures the key is present in the ordered key list so newly-created items become
    /// visible even when an optimistic placeholder was inserted earlier.
    ///
    /// If the server may have assigned a different key than the one used in the optimistic
    /// placeholder (a typical create flow with a temporary client-side id), use
    /// [`update_item_with_old_key`](Self::update_item_with_old_key) instead so the placeholder
    /// gets re-keyed in place rather than leaving an orphan under the old key.
    pub fn update_item(&self, item: T::Value) {
        let key = T::get_key(&item);
        if self
            .items
            .must_change(&key, |list_item| {
                list_item.original.set(item.clone());
                list_item.override_val.set(None);
            })
            .is_none()
        {
            // Key not yet in cache — insert it as a committed original.
            self.items.insert(key.clone(), ListItem::new(item));
        }
        self.optimistic_placeholders.remove(&key);
        self.keys.change(|current_keys| {
            if !current_keys.contains(&key) {
                current_keys.push(key);
            }
        });
    }

    /// Like [`update_item`](Self::update_item), but for cases where the server assigns a different
    /// key than the one used in the optimistic placeholder (typical create flow with a temporary
    /// client-side id). The entry is re-keyed in place: the position of `old_key` in the ordered
    /// key list is preserved, and any optimistic override under `old_key` is cleared.
    ///
    /// If `old_key` and `T::get_key(&item)` are equal this delegates to [`update_item`](Self::update_item).
    pub fn update_item_with_old_key(&self, old_key: &T::Key, item: T::Value) {
        let new_key = T::get_key(&item);
        if &new_key == old_key {
            self.update_item(item);
            return;
        }
        // Drop the placeholder under old_key.
        self.items.remove(old_key);
        self.optimistic_placeholders.remove(old_key);
        // Insert (or update) under the new key.
        if self
            .items
            .must_change(&new_key, |list_item| {
                list_item.original.set(item.clone());
                list_item.override_val.set(None);
            })
            .is_none()
        {
            self.items.insert(new_key.clone(), ListItem::new(item));
        }
        self.optimistic_placeholders.remove(&new_key);
        // Replace old_key with new_key in the ordered keys list, preserving position.
        self.keys.change(|keys| {
            let old_pos = keys.iter().position(|k| k == old_key);
            let already_has_new = keys.contains(&new_key);
            match (old_pos, already_has_new) {
                (Some(pos), false) => keys[pos] = new_key.clone(),
                (Some(pos), true) => {
                    keys.remove(pos);
                }
                (None, false) => keys.push(new_key.clone()),
                (None, true) => {}
            }
        });
    }

    /// Remove a single item from the cache: drops it from `items`, from the ordered key list,
    /// and clears any placeholder marker. Typically called after a successful `DELETE` to confirm
    /// a prior [`optimistically_remove_item`](Self::optimistically_remove_item).
    pub fn remove_item(&self, key: &T::Key) {
        self.items.remove(key);
        self.optimistic_placeholders.remove(key);
        self.keys
            .change(|current_keys| current_keys.retain(|k| k != key));
    }

    /// Commit the current override into the original value without contacting the server.
    ///
    /// - If the override holds a value, it becomes the new `original` and the override is cleared.
    /// - If the override marks the item as removed (`Some(None)`), the entry is dropped from
    ///   both `items` and the ordered key list.
    /// - If there is no active override, this is a no-op.
    ///
    /// Either way the placeholder marker is cleared, since a successful commit is treated as
    /// confirmation — a subsequent [`rollback`](Self::rollback) on the same key will only clear
    /// overrides, not delete the row.
    pub fn commit(&self, key: &T::Key) {
        self.items.must_change(key, |list_item| {
            transaction(|ctx| {
                if let Some(override_val) = list_item.override_val.get(ctx) {
                    if let Some(override_val) = override_val {
                        list_item.original.set(override_val.clone());
                        list_item.override_val.set(None);
                    } else {
                        self.items.remove(key);
                        self.keys.change(|keys| keys.retain(|k| k != key));
                    }
                }
            });
        });
        // Treat a successful commit as confirmation, dropping the placeholder marker.
        self.optimistic_placeholders.remove(key);
    }

    /// Discard the optimistic override for a key, restoring the original server value.
    /// If the key was inserted purely optimistically (no server confirmation has ever been
    /// recorded), the entry is fully removed from the list — both `items` and `keys` — so a
    /// failed create doesn't leave its draft visible.
    pub fn rollback(&self, key: &T::Key) {
        if self.optimistic_placeholders.remove(key).is_some() {
            self.items.remove(key);
            self.keys
                .change(|current_keys| current_keys.retain(|k| k != key));
        } else {
            self.items
                .must_change(key, |list_item| list_item.override_val.set(None));
        }
    }

    pub fn to_computed(&self) -> Computed<Resource<Rc<Vec<T::Value>>>> {
        let state = self.clone();
        Computed::from(move |context| state.get(context))
    }

    pub fn render(&self, render: impl Fn(Rc<Vec<T::Value>>) -> DomNode + 'static) -> DomNode {
        self.to_computed().render_value(move |value| match value {
            Resource::Ready(value) => render(value),
            Resource::Loading => {
                use crate as vertigo;
                vertigo::dom! { <div>"Loading ..."</div> }
            }
            Resource::Error(error) => {
                use crate as vertigo;
                vertigo::dom! { <div>"error = " {error}</div> }
            }
        })
    }

    fn update(&self, with_loading: bool, force: bool) {
        if self.list_queued.get() {
            return;
        }
        self.list_queued.set(true);

        if with_loading {
            self.state.set(ListCacheState::Loading);
        }

        let self_clone = self.clone();

        get_driver().spawn(async move {
            let state = transaction(|ctx| self_clone.state.get(ctx));
            let needs_fetch = force || state == ListCacheState::Uninitialized;

            if needs_fetch {
                self_clone.state.set(ListCacheState::Loading);

                let request = transaction(|context| {
                    self_clone
                        .list_request
                        .as_ref()
                        .clone()
                        .to_request_context(context)
                });

                let result = api_fetch().fetch(request.clone()).await;
                let new_value = RequestResponse::new(request, result)
                    .into(self_clone.list_map_response.as_ref());

                match new_value {
                    Ok(items_vec) => {
                        self_clone.apply_response(items_vec);
                        self_clone.state.set(ListCacheState::Ready);
                    }
                    Err(msg) => {
                        self_clone.state.set(ListCacheState::Error(msg));
                    }
                }
            }

            self_clone.list_queued.set(false);
        });
    }

    fn apply_response(&self, new_items: Vec<T::Value>) {
        let new_keys: Vec<T::Key> = new_items.iter().map(T::get_key).collect();
        let new_key_set: HashSet<T::Key> = new_keys.iter().cloned().collect();

        // Remove items no longer present in the response.
        self.items.retain(|k, _| new_key_set.contains(k));

        // Update originals for existing items; insert new ones.
        for item in new_items {
            let key = T::get_key(&item);
            if self
                .items
                .must_change(&key, |list_item| list_item.original.set(item.clone()))
                .is_none()
            {
                self.items.insert(key, ListItem::new(item));
            }
        }

        // Server response is the source of truth — every key it returns is now confirmed,
        // and any unreturned placeholder has just been dropped above. Either way, no entry
        // remains a placeholder after a refresh.
        self.optimistic_placeholders.retain(|_, _| false);

        self.keys.set(new_keys);
    }
}

impl<T: CollectionKey> LazyListCache<T>
where
    T::Value: JsJsonDeserialize,
{
    /// Create a `LazyListCache` that deserializes `Vec<T::Value>` from the given URL.
    pub fn new_resource(api: &str, path: &str, ttl: u64) -> Self {
        let url = [api, path].concat();
        LazyListCache::new(
            get_driver().request_get(url).ttl_seconds(ttl),
            |status, body| {
                if status == 200 {
                    Some(body.into::<Vec<T::Value>>())
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
    use crate::{Resource, transaction};

    #[derive(Clone, PartialEq, Debug)]
    struct Item {
        id: i32,
        name: String,
    }

    struct ItemKey;

    impl CollectionKey for ItemKey {
        type Key = i32;
        type Value = Item;

        fn get_key(val: &Item) -> i32 {
            val.id
        }
    }

    fn make_cache() -> LazyListCache<ItemKey> {
        LazyListCache::new(RequestBuilder::get("https://test.example/items"), |_, _| {
            None
        })
    }

    fn seed(cache: &LazyListCache<ItemKey>) {
        cache.apply_response(vec![
            Item {
                id: 1,
                name: "One".to_string(),
            },
            Item {
                id: 2,
                name: "Two".to_string(),
            },
            Item {
                id: 3,
                name: "Three".to_string(),
            },
        ]);
        cache.state.set(ListCacheState::Ready);
    }

    fn get_by_key(cache: &LazyListCache<ItemKey>, key: i32) -> Resource<Rc<Item>> {
        transaction(|ctx| cache.get_by_key(ctx, &key))
    }

    fn get_list(cache: &LazyListCache<ItemKey>) -> Rc<Vec<Item>> {
        transaction(|ctx| match cache.get(ctx) {
            Resource::Ready(v) => v,
            other => panic!("expected Ready, got {:?}", other),
        })
    }

    // --- get ---

    #[test]
    fn test_get_returns_ordered_originals() {
        let cache = make_cache();
        seed(&cache);

        let list = get_list(&cache);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].id, 1);
        assert_eq!(list[1].id, 2);
        assert_eq!(list[2].id, 3);
    }

    #[test]
    fn test_get_returns_loading_before_fetch() {
        let cache = make_cache();
        transaction(|ctx| assert_eq!(cache.get(ctx), Resource::Loading));
    }

    // --- get_by_key ---

    #[test]
    fn test_get_by_key_falls_back_to_original() {
        let cache = make_cache();
        seed(&cache);

        assert_eq!(
            get_by_key(&cache, 2),
            Resource::Ready(Rc::new(Item {
                id: 2,
                name: "Two".to_string()
            }))
        );
    }

    #[test]
    fn test_get_by_key_loading_when_not_ready() {
        let cache = make_cache();
        assert_eq!(get_by_key(&cache, 1), Resource::Loading);
    }

    #[test]
    fn test_get_by_key_error_when_key_missing() {
        let cache = make_cache();
        seed(&cache);
        assert!(matches!(get_by_key(&cache, 99), Resource::Error(_)));
    }

    // --- optimistically_set_item ---

    #[test]
    fn test_override_visible_via_get_by_key() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "Two-override".to_string(),
        });

        assert_eq!(
            get_by_key(&cache, 2),
            Resource::Ready(Rc::new(Item {
                id: 2,
                name: "Two-override".to_string()
            }))
        );
    }

    #[test]
    fn test_override_does_not_modify_original() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "Two-override".to_string(),
        });

        // get() still shows the original
        let list = get_list(&cache);
        assert_eq!(list[1].name, "Two");
    }

    #[test]
    fn test_override_other_keys_unaffected() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "Two-override".to_string(),
        });

        assert_eq!(
            get_by_key(&cache, 1),
            Resource::Ready(Rc::new(Item {
                id: 1,
                name: "One".to_string()
            }))
        );
        assert_eq!(
            get_by_key(&cache, 3),
            Resource::Ready(Rc::new(Item {
                id: 3,
                name: "Three".to_string()
            }))
        );
    }

    #[test]
    fn test_override_works_while_loading() {
        let cache = make_cache();

        cache.optimistically_set_item(Item {
            id: 1,
            name: "One".to_string(),
        });

        assert_eq!(
            get_by_key(&cache, 1),
            Resource::Ready(Rc::new(Item {
                id: 1,
                name: "One".to_string()
            }))
        );
        // full list is still Loading
        transaction(|ctx| assert_eq!(cache.get(ctx), Resource::Loading));
    }

    #[test]
    fn test_override_update_same_key() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "v1".to_string(),
        });
        cache.optimistically_set_item(Item {
            id: 2,
            name: "v2".to_string(),
        });

        assert_eq!(
            get_by_key(&cache, 2),
            Resource::Ready(Rc::new(Item {
                id: 2,
                name: "v2".to_string()
            }))
        );
    }

    // --- rollback ---

    #[test]
    fn test_rollback_restores_original() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "Two-override".to_string(),
        });
        cache.rollback(&2);

        assert_eq!(
            get_by_key(&cache, 2),
            Resource::Ready(Rc::new(Item {
                id: 2,
                name: "Two".to_string()
            }))
        );
    }

    #[test]
    fn test_rollback_noop_for_unknown_key() {
        let cache = make_cache();
        seed(&cache);

        cache.rollback(&99); // should not panic

        let list = get_list(&cache);
        assert_eq!(list.len(), 3);
    }

    // --- apply_response (refresh preserves overrides) ---

    #[test]
    fn test_refresh_preserves_override() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "Two-override".to_string(),
        });

        // Server returns updated data
        cache.apply_response(vec![
            Item {
                id: 1,
                name: "One-v2".to_string(),
            },
            Item {
                id: 2,
                name: "Two-v2".to_string(),
            },
            Item {
                id: 3,
                name: "Three-v2".to_string(),
            },
        ]);
        cache.state.set(ListCacheState::Ready);

        // get() shows server originals
        let list = get_list(&cache);
        assert_eq!(list[1].name, "Two-v2");

        // get_by_key still shows override
        assert_eq!(
            get_by_key(&cache, 2),
            Resource::Ready(Rc::new(Item {
                id: 2,
                name: "Two-override".to_string()
            }))
        );
    }

    // --- create flow (optimistic insert + update_item commit) ---

    #[test]
    fn test_optimistic_set_then_update_item_appears_in_list() {
        // Mirrors the panel's create-peer flow: optimistic insert, then commit on POST 200.
        let cache = make_cache();
        seed(&cache);

        let new_item = Item {
            id: 42,
            name: "New".to_string(),
        };

        cache.optimistically_set_item(new_item.clone());

        // Optimistic placeholder is already visible via get() (placeholder original = optimistic).
        let list = get_list(&cache);
        assert!(list.iter().any(|i| i.id == 42 && i.name == "New"));

        // Server confirms — commit the optimistic insert.
        cache.update_item(new_item.clone());

        let list = get_list(&cache);
        assert_eq!(list.len(), 4);
        assert!(list.iter().any(|i| i.id == 42 && i.name == "New"));

        // After update_item, the override is cleared so get_by_key returns the committed original.
        assert_eq!(get_by_key(&cache, 42), Resource::Ready(Rc::new(new_item)));
    }

    #[test]
    fn test_update_item_inserts_unknown_key() {
        let cache = make_cache();
        seed(&cache);

        cache.update_item(Item {
            id: 99,
            name: "Ninety-nine".to_string(),
        });

        let list = get_list(&cache);
        assert_eq!(list.len(), 4);
        assert!(list.iter().any(|i| i.id == 99));
    }

    #[test]
    fn test_keys_reactivity_via_computed() {
        // Subscribers wrapped in a Computed must re-evaluate when keys change.
        let cache = make_cache();
        seed(&cache);

        let computed = {
            let cache = cache.clone();
            Computed::from(move |ctx| match cache.get(ctx) {
                Resource::Ready(v) => v.iter().map(|i| i.id).collect::<Vec<_>>(),
                _ => Vec::new(),
            })
        };

        let initial = transaction(|ctx| computed.get(ctx));
        assert_eq!(initial, vec![1, 2, 3]);

        cache.update_item(Item {
            id: 4,
            name: "Four".to_string(),
        });

        let after = transaction(|ctx| computed.get(ctx));
        assert_eq!(after, vec![1, 2, 3, 4]);
    }

    // --- update_item_with_old_key (re-keying create flow) ---

    #[test]
    fn test_update_item_with_old_key_rekeys_in_place() {
        // Mirrors a typical create flow: optimistic insert under a temp id (0), then the server
        // returns the created entity with a real id (42). The placeholder should be re-keyed to
        // 42 at the same position in the list, and lookups by 0 should fail.
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 0,
            name: "draft".to_string(),
        });

        // Optimistic placeholder is appended at the end: [1, 2, 3, 0].
        let list = get_list(&cache);
        assert_eq!(
            list.iter().map(|i| i.id).collect::<Vec<_>>(),
            vec![1, 2, 3, 0]
        );

        cache.update_item_with_old_key(
            &0,
            Item {
                id: 42,
                name: "Saved".to_string(),
            },
        );

        // Position is preserved: [1, 2, 3, 42], not [1, 2, 3, 0, 42].
        let list = get_list(&cache);
        assert_eq!(
            list.iter().map(|i| i.id).collect::<Vec<_>>(),
            vec![1, 2, 3, 42]
        );
        assert_eq!(list[3].name, "Saved");

        // Old key is gone.
        assert!(matches!(get_by_key(&cache, 0), Resource::Error(_)));
        // New key resolves with the committed original.
        assert_eq!(
            get_by_key(&cache, 42),
            Resource::Ready(Rc::new(Item {
                id: 42,
                name: "Saved".to_string()
            }))
        );
    }

    #[test]
    fn test_update_item_with_old_key_same_key_acts_as_update() {
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "draft".to_string(),
        });
        cache.update_item_with_old_key(
            &2,
            Item {
                id: 2,
                name: "Saved".to_string(),
            },
        );

        let list = get_list(&cache);
        assert_eq!(list.len(), 3);
        assert_eq!(list[1].name, "Saved");
    }

    // --- rollback of optimistic create (placeholder) ---

    #[test]
    fn test_rollback_removes_optimistic_placeholder() {
        // A failed optimistic create must not leave a draft visible in the list.
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 99,
            name: "draft".to_string(),
        });

        // Visible during the optimistic phase.
        let list = get_list(&cache);
        assert!(list.iter().any(|i| i.id == 99));

        cache.rollback(&99);

        // Fully gone.
        let list = get_list(&cache);
        assert_eq!(list.len(), 3);
        assert!(!list.iter().any(|i| i.id == 99));
        assert!(matches!(get_by_key(&cache, 99), Resource::Error(_)));
    }

    #[test]
    fn test_rollback_existing_item_only_clears_override() {
        // For server-confirmed items, rollback must continue to behave as before:
        // clear the override, restore the original, but never delete the row.
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_set_item(Item {
            id: 2,
            name: "Two-override".to_string(),
        });
        cache.rollback(&2);

        let list = get_list(&cache);
        assert_eq!(list.len(), 3);
        assert_eq!(list[1].name, "Two");
    }

    // --- delete flow (remove_item / optimistically_remove_item) ---

    #[test]
    fn test_optimistic_then_remove_item_reactivity_via_granular() {
        // Full delete flow: optimistically_remove_item then, post-await, remove_item.
        // Verify the granular consumer sees the row disappear.
        let cache = make_cache();
        seed(&cache);

        let computed = {
            let cache = cache.clone();
            Computed::from(move |ctx| {
                let granular = cache.granular::<fn(&Item) -> bool>(ctx, None);
                match granular {
                    Resource::Ready(items) => items
                        .iter()
                        .filter_map(|item| item.get(ctx).map(|v| v.id))
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                }
            })
        };

        assert_eq!(transaction(|ctx| computed.get(ctx)), vec![1, 2, 3]);

        cache.optimistically_remove_item(&2);
        // During optimistic phase, granular still shows the row (granular ignores override_val).
        assert_eq!(transaction(|ctx| computed.get(ctx)), vec![1, 2, 3]);

        cache.remove_item(&2);
        assert_eq!(transaction(|ctx| computed.get(ctx)), vec![1, 3]);
    }

    #[test]
    fn test_remove_item_after_optimistically_remove_item() {
        // The full delete flow: optimistic mark, then commit on server success.
        let cache = make_cache();
        seed(&cache);

        cache.optimistically_remove_item(&2);
        cache.remove_item(&2);

        let list = get_list(&cache);
        assert_eq!(list.iter().map(|i| i.id).collect::<Vec<_>>(), vec![1, 3]);
        assert!(matches!(get_by_key(&cache, 2), Resource::Error(_)));
    }

    #[test]
    fn test_update_item_clears_placeholder_so_later_rollback_keeps_row() {
        // After update_item commits an optimistic create, a subsequent rollback (e.g. on a
        // follow-up edit) must NOT delete the now-confirmed row.
        let cache = make_cache();
        seed(&cache);

        let new_item = Item {
            id: 99,
            name: "draft".to_string(),
        };
        cache.optimistically_set_item(new_item.clone());
        cache.update_item(Item {
            id: 99,
            name: "Saved".to_string(),
        });

        // User edits optimistically and then cancels.
        cache.optimistically_set_item(Item {
            id: 99,
            name: "draft-edit".to_string(),
        });
        cache.rollback(&99);

        let list = get_list(&cache);
        assert_eq!(list.len(), 4);
        assert!(list.iter().any(|i| i.id == 99 && i.name == "Saved"));
    }

    // --- clone / PartialEq ---

    #[test]
    fn test_clone_eq() {
        let cache = make_cache();
        assert_eq!(cache, cache.clone());
        assert_ne!(cache, make_cache());
    }
}
