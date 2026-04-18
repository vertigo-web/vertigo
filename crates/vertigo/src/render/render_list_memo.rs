use std::rc::Rc;
use vertigo_macro::bind;

use crate::{
    Computed, DomNode, DropResource, LazyCache, Value,
    render::{collection::Collection, render_list},
};

/// Renders a reactive list from a `Value<Rc<Vec<T::Value>>>`, memoizing each item.
///
/// So that only items whose values actually changed are re-rendered. The list
/// automatically stays in sync with the source value and cleans up when dropped.
pub fn render_list_memo<T: crate::render::collection::CollectionKey + 'static>(
    value: &Value<Rc<Vec<T::Value>>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode {
    let (collection, drop_synchronize) = value.synchronize::<Collection<T>>();

    let computed = collection.get();

    let result = render_list(
        computed,
        |item| item.key.clone(),
        move |item| render(&item.model),
    );

    result.append_drop_resource(drop_synchronize);

    result.append_drop_resource(DropResource::new(bind!(value, || {
        drop(value);
    })));

    result
}

/// Renders a reactive list from a `LazyCache<Vec<T::Value>>`, memoizing each item.
///
/// So that only items whose values actually changed are re-rendered. Unlike
/// `render_list_memo`, the source is a lazily-loaded cache (e.g. fetched from a
/// remote resource), and the list updates whenever the cache is refreshed.
pub fn render_resource_list_memo<T: crate::render::collection::CollectionKey + 'static>(
    value: &LazyCache<Vec<T::Value>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode {
    let (collection, drop_event) = value.synchronize::<Collection<T>>();

    let computed = collection.get();

    let result = render_list(
        computed,
        |item| item.key.clone(),
        move |item| render(&item.model),
    );

    result.append_drop_resource(drop_event);

    result.append_drop_resource(DropResource::new(bind!(value, || {
        drop(value);
    })));

    result
}
