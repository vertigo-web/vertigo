use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use super::{Computed, ToComputed, struct_mut::ValueMut};

/// One entry in a [`keyed_computed_list`]: a stable key plus a per-item value.
///
/// For [`keyed_computed_list`] itself, `V` is [`Computed<T>`]. `PartialEq` then
/// compares the key and the identity of that `Computed` (not the item value), so
/// observers of the outer list ignore in-place item updates.
pub struct KeyedListItem<K, V> {
    pub key: K,
    pub value: V,
}

impl<K: Clone, V: Clone> Clone for KeyedListItem<K, V> {
    fn clone(&self) -> Self {
        KeyedListItem {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for KeyedListItem<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

/// Maps a reactive list into a reactive list of per-item [`Computed`]s, reusing the same
/// `Computed` instance for each key across updates (Solid `<For>`-style).
///
/// The **outer** computed changes when membership or order changes. Each **inner**
/// `Computed` changes only when that item's value changes (`T: PartialEq`). Duplicate
/// keys are logged and skipped (the first occurrence is kept).
///
/// If a per-item `Computed` is read after its key has left the source list, the last
/// seen value is returned. Drop observers when a row unmounts.
///
/// This is the `Computed`-to-`Computed` transform used by
/// [`render_list`](crate::render::render_list) /
/// [`render_list_memo`](crate::render::render_list_memo).
///
/// ```rust
/// use vertigo::{keyed_computed_list, transaction, Value};
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct Person {
///     id: u32,
///     name: String,
/// }
///
/// let people = Value::new(vec![Person {
///     id: 1,
///     name: "Ann".into(),
/// }]);
///
/// let rows = keyed_computed_list(people.to_computed(), |person| person.id);
///
/// transaction(|ctx| {
///     let list = rows.get(ctx);
///     assert_eq!(list.len(), 1);
///     assert_eq!(list[0].key, 1);
///     assert_eq!(list[0].value.get(ctx).name, "Ann");
/// });
/// ```
pub fn keyed_computed_list<T, K>(
    items: impl ToComputed<Vec<T>>,
    get_key: impl Fn(&T) -> K + 'static,
) -> Computed<Vec<KeyedListItem<K, Computed<T>>>>
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + std::fmt::Debug + 'static,
{
    let items = items.to_computed();
    let get_key = Rc::new(get_key);

    // Behind an `Rc` for the same reason as `hash` below: both readers would otherwise
    // deep-copy the whole list on every update.
    let unique_keyed_items = Computed::from({
        move |ctx| {
            let mut result = Vec::new();
            let mut seen = HashSet::new();

            for item in items.get(ctx) {
                let key = get_key(&item);

                if seen.contains(&key) {
                    log::error!(
                        "keyed_computed_list: duplicate key {:?}; keeping the first occurrence",
                        key
                    );
                    continue;
                }

                seen.insert(key.clone());
                result.push((key, item));
            }

            Rc::new(result)
        }
    });

    // Rows are looked up by key from here. A row only notifies when its own value
    // changes, because `Computed` compares with `PartialEq` before notifying.
    //
    // Behind an `Rc` because every row reads this map, and `Computed::get` hands back a
    // clone of the cached value - cloning the map itself would make one update cost
    // `rows * rows` item clones.
    let hash = Computed::from({
        let unique_keyed_items = unique_keyed_items.clone();
        move |ctx| {
            Rc::new(
                unique_keyed_items
                    .get(ctx)
                    .iter()
                    .map(|(key, item)| (key.clone(), item.clone()))
                    .collect::<HashMap<K, T>>(),
            )
        }
    });

    let cache_computed = Rc::new(ValueMut::new(HashMap::<K, Computed<T>>::new()));
    let cache_list_items = Rc::new(ValueMut::new(
        HashMap::<K, KeyedListItem<K, Computed<T>>>::new(),
    ));

    Computed::from({
        move |ctx| {
            let unique_items = unique_keyed_items.get(ctx);
            let mut result_list = Vec::with_capacity(unique_items.len());

            for (key, item) in unique_items.iter() {
                let next_computed = cache_computed.change(|cache| {
                    if let Some(prev) = cache.get(key) {
                        prev.clone()
                    } else {
                        let hash = hash.clone();
                        let last = Rc::new(ValueMut::new(item.clone()));
                        let lookup_key = key.clone();
                        Computed::from(move |ctx| match hash.get(ctx).get(&lookup_key) {
                            Some(val) => {
                                let val = val.clone();
                                last.set(val.clone());
                                val
                            }
                            None => {
                                log::error!(
                                    "keyed_computed_list: item Computed for key {:?} was read after that key left the source list; returning last value",
                                    lookup_key
                                );
                                last.get()
                            }
                        })
                    }
                });

                let list_item = cache_list_items.change(|cache| match cache.get(key) {
                    Some(prev) if prev.value == next_computed => prev.clone(),
                    _ => KeyedListItem {
                        key: key.clone(),
                        value: next_computed.clone(),
                    },
                });

                result_list.push(list_item);
            }

            cache_computed.set(
                result_list
                    .iter()
                    .map(|item| (item.key.clone(), item.value.clone()))
                    .collect(),
            );
            cache_list_items.set(
                result_list
                    .iter()
                    .map(|item| (item.key.clone(), item.clone()))
                    .collect(),
            );

            result_list
        }
    })
}
