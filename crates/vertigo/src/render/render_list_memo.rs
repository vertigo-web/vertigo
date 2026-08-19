use std::rc::Rc;

use crate::{
    Computed, DomNode, LazyCache, Resource, Value,
    render::{collection::CollectionKey, render_list},
};

/// Renders a reactive list from a `Value<Rc<Vec<T::Value>>>`, memoizing each item.
///
/// Thin wrapper around [`render_list`]: maps `Rc<Vec<_>>` to `Vec` and uses
/// [`CollectionKey`](crate::CollectionKey) for identity. `render` is called once
/// per key with that item's stable [`Computed`](crate::Computed).
pub fn render_list_memo<T: CollectionKey + 'static>(
    value: &Value<Rc<Vec<T::Value>>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode {
    let items = value.to_computed().map(Rc::unwrap_or_clone);
    render_list(items, T::get_key, render)
}

/// Renders a reactive list from a `LazyCache<Vec<T::Value>>`, memoizing each item.
///
/// So that only items whose values actually changed are re-rendered. Unlike
/// `render_list_memo`, the source is a lazily-loaded cache (e.g. fetched from a
/// remote resource), and the list updates whenever the cache is refreshed.
///
/// Loading / Error states are treated as an empty list.
pub fn render_resource_list_memo<T: CollectionKey + 'static>(
    value: &LazyCache<Vec<T::Value>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode {
    let items = value.to_computed().map(|resource| match resource {
        Resource::Ready(list) => Rc::unwrap_or_clone(list),
        Resource::Loading | Resource::Error(_) => Vec::new(),
    });
    render_list(items, T::get_key, render)
}
