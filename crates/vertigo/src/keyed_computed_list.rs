use std::{cell::Cell, collections::hash_map::Entry, hash::Hash, rc::Rc};

use crate::{Computed, ToComputed, fast_hash::FastMap, struct_mut::ValueMut};

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

/// Every key ever seen, with the `Computed` handed out for it and the pass that last saw it.
type RowCache<K, T> = FastMap<K, (u64, Computed<T>)>;

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
///
/// # Cost of an update
///
/// One update of an n-row list is O(n) whatever changed, and that is inherent here: the
/// source is a `Computed<Vec<T>>`, so reading it copies the vector, and the order has to be
/// walked to be diffed. What the three nodes below are shaped for is keeping the constant
/// small - one hash lookup, two key clones and one item clone per row, rather than a
/// rebuild of every index.
pub fn keyed_computed_list<T, K>(
    items: impl ToComputed<Vec<T>>,
    get_key: impl Fn(&T) -> K + 'static,
) -> Computed<Vec<KeyedListItem<K, Computed<T>>>>
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + std::fmt::Debug + 'static,
{
    let items = items.to_computed();

    // Order and lookup, built together in one pass. The map is also the duplicate detector,
    // so there is no separate set, and each item is moved into it rather than copied into a
    // second structure.
    let indexed = Computed::from({
        move |ctx| {
            let items = items.get(ctx);

            let mut order = Vec::with_capacity(items.len());
            let mut by_key = FastMap::with_capacity_and_hasher(items.len(), Default::default());

            for item in items {
                let key = get_key(&item);

                match by_key.entry(key.clone()) {
                    Entry::Occupied(_) => {
                        log::error!(
                            "keyed_computed_list: duplicate key {:?}; keeping the first occurrence",
                            key
                        );
                    }
                    Entry::Vacant(slot) => {
                        slot.insert(item);
                        order.push(key);
                    }
                }
            }

            (Rc::new(order), Rc::new(by_key))
        }
    });

    // The lookup on its own, and deliberately its own node.
    //
    // Every row reads this, so its equality cutoff is what decides whether n rows recompute.
    // Reordering a list leaves the map untouched, so a swap stops here; folding this back
    // into `indexed` - whose value also carries the order - would make every reorder re-run
    // the whole list. `unchanged_rows_do_not_notify` is the test that holds this in place.
    let by_key = Computed::from({
        let indexed = indexed.clone();
        move |ctx| indexed.get(ctx).1
    });

    // Key -> the `Computed` handed out for it, stamped with the pass that last saw the key.
    // The stamp is what lets a single `retain` evict departed keys without a second set:
    // re-running a pass simply re-stamps, so a repeated run is harmless.
    let cache: Rc<ValueMut<RowCache<K, T>>> = Rc::new(ValueMut::new(FastMap::default()));
    let pass = Rc::new(Cell::new(0u64));

    Computed::from({
        move |ctx| {
            let (order, by_key_now) = indexed.get(ctx);

            let stamp = pass.get().wrapping_add(1);
            pass.set(stamp);

            let mut result_list = Vec::with_capacity(order.len());

            cache.change(|cache| {
                for key in order.iter() {
                    let value = match cache.get_mut(key) {
                        Some(entry) => {
                            entry.0 = stamp;
                            entry.1.clone()
                        }
                        None => {
                            // `order` and the map were built from the same pass, so every
                            // key here is in it. Skipping rather than inventing a value is
                            // what a future edit that breaks that should cost.
                            let Some(item) = by_key_now.get(key) else {
                                continue;
                            };

                            let value = row_computed(
                                key.clone(),
                                &by_key,
                                by_key_now.clone(),
                                item.clone(),
                            );
                            cache.insert(key.clone(), (stamp, value.clone()));
                            value
                        }
                    };

                    result_list.push(KeyedListItem {
                        key: key.clone(),
                        value,
                    });
                }

                cache.retain(|_, (seen, _)| *seen == stamp);
            });

            result_list
        }
    })
}

/// The `Computed` for one key: look the key up in the shared map, and hold on to the map
/// that answered.
///
/// `last` keeps the *map* rather than the value, which is what makes the ordinary path cost
/// one item clone instead of two - retaining the `Rc` is a pointer bump, retaining the value
/// is a copy of it. Every row re-runs whenever a key joins or leaves the list, so that clone
/// is paid n times per membership change and is worth removing.
///
/// The map doubles as the answer for a read after the key has left: the retained map is the
/// last one that still had it. `seed` covers the one case it cannot - a read that happens
/// before this `Computed` has ever run.
fn row_computed<T, K>(
    key: K,
    by_key: &Computed<Rc<FastMap<K, T>>>,
    initial: Rc<FastMap<K, T>>,
    seed: T,
) -> Computed<T>
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + std::fmt::Debug + 'static,
{
    let by_key = by_key.clone();
    let last = Rc::new(ValueMut::new(initial));

    Computed::from(move |ctx| {
        let current = by_key.get(ctx);

        if let Some(value) = current.get(&key) {
            let value = value.clone();
            last.set(current);
            return value;
        }

        log::error!(
            "keyed_computed_list: item Computed for key {:?} was read after that key left the source list; returning last value",
            key
        );

        let value = last
            .map(|last| last.get(&key).cloned())
            .unwrap_or_else(|| seed.clone());

        // The key is gone for good, so shrink what this row retains to its own last value.
        // Holding the whole map is right while the row is live - every row shares that one
        // `Rc` - but a row nobody re-renders would otherwise pin every item of the list it
        // was last part of.
        last.change(|last| {
            if last.len() > 1 {
                *last = Rc::new(FastMap::from_iter([(key.clone(), value.clone())]));
            }
        });

        value
    })
}
