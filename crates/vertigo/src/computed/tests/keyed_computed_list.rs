use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    Computed, DropResource, KeyedListItem, Value, computed::struct_mut::ValueMut,
    keyed_computed_list, transaction,
};

#[derive(Clone, PartialEq, Debug)]
struct Person {
    id: &'static str,
    name: &'static str,
    age: i32,
}

fn bob() -> Person {
    Person {
        id: "1",
        name: "Bob",
        age: 43,
    }
}

fn frank(age: i32) -> Person {
    Person {
        id: "2",
        name: "Frank",
        age,
    }
}

/// Like a JS `Signal.set(newArray)`: every assignment notifies, even when the
/// payload is structurally equal. `Value<Vec<_>>` would swallow that case via `PartialEq`.
#[derive(Clone)]
struct SignalList(Vec<Person>);

impl PartialEq for SignalList {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CleanDumpItem {
    id: &'static str,
    name: &'static str,
    age: i32,
    revision: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct CleanDump {
    list_revision: u32,
    items: Vec<CleanDumpItem>,
}

struct DumpItem {
    id: &'static str,
    name: RefCell<&'static str>,
    age: RefCell<i32>,
    revision: RefCell<u32>,
    _unsubscribe: RefCell<Option<DropResource>>,
}

struct Dump {
    list_revision: u32,
    items: Vec<Rc<DumpItem>>,
}

/// Mirrors the TypeScript `autorun` dump: subscribe to the outer list, and for each
/// new key start a nested subscribe on that row's `Computed`.
fn watch_dump(
    list: Computed<Vec<KeyedListItem<&'static str, Computed<Person>>>>,
) -> (Rc<RefCell<Dump>>, DropResource) {
    let dump = Rc::new(RefCell::new(Dump {
        list_revision: 0,
        items: Vec::new(),
    }));

    let unsub = list.subscribe({
        let dump = dump.clone();
        move |rows| {
            let mut dump = dump.borrow_mut();

            let mut prev: HashMap<&'static str, Rc<DumpItem>> = HashMap::new();
            for item in dump.items.drain(..) {
                prev.insert(item.id, item);
            }

            let mut new_items = Vec::new();
            for record in rows {
                let id = record.key;
                if let Some(prev_item) = prev.remove(&id) {
                    new_items.push(prev_item);
                    continue;
                }

                let new_item = Rc::new(DumpItem {
                    id,
                    name: RefCell::new(""),
                    age: RefCell::new(0),
                    revision: RefCell::new(0),
                    _unsubscribe: RefCell::new(None),
                });

                let item_unsub = record.value.subscribe({
                    let new_item = new_item.clone();
                    move |person| {
                        *new_item.name.borrow_mut() = person.name;
                        *new_item.age.borrow_mut() = person.age;
                        *new_item.revision.borrow_mut() += 1;
                    }
                });
                *new_item._unsubscribe.borrow_mut() = Some(item_unsub);
                new_items.push(new_item);
            }

            dump.items = new_items;
            dump.list_revision += 1;
        }
    });

    (dump, unsub)
}

fn get_dump(dump: &RefCell<Dump>) -> CleanDump {
    let dump = dump.borrow();
    CleanDump {
        list_revision: dump.list_revision,
        items: dump
            .items
            .iter()
            .map(|item| CleanDumpItem {
                id: item.id,
                name: *item.name.borrow(),
                age: *item.age.borrow(),
                revision: *item.revision.borrow(),
            })
            .collect(),
    }
}

fn people_computed(source: &Value<SignalList>) -> Computed<Vec<Person>> {
    let source = source.clone();
    Computed::from(move |ctx| source.get(ctx).0)
}

#[test]
fn exposes_computed_values_for_the_initial_list() {
    let source = Value::new(SignalList(Vec::new()));
    let list = keyed_computed_list(people_computed(&source), |item| item.id);

    let (dump, _watch) = watch_dump(list);

    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 1,
            items: vec![],
        }
    );

    source.set(SignalList(vec![bob()]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 2,
            items: vec![CleanDumpItem {
                id: "1",
                name: "Bob",
                age: 43,
                revision: 1,
            }],
        }
    );

    // New array, same content — source notifies, keyed list must not.
    source.set(SignalList(vec![bob()]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 2,
            items: vec![CleanDumpItem {
                id: "1",
                name: "Bob",
                age: 43,
                revision: 1,
            }],
        }
    );

    source.set(SignalList(vec![bob(), frank(23)]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 3,
            items: vec![
                CleanDumpItem {
                    id: "1",
                    name: "Bob",
                    age: 43,
                    revision: 1,
                },
                CleanDumpItem {
                    id: "2",
                    name: "Frank",
                    age: 23,
                    revision: 1,
                },
            ],
        }
    );

    source.set(SignalList(vec![bob(), frank(24)]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 3,
            items: vec![
                CleanDumpItem {
                    id: "1",
                    name: "Bob",
                    age: 43,
                    revision: 1,
                },
                CleanDumpItem {
                    id: "2",
                    name: "Frank",
                    age: 24,
                    revision: 2,
                },
            ],
        }
    );

    source.set(SignalList(vec![frank(24)]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 4,
            items: vec![CleanDumpItem {
                id: "2",
                name: "Frank",
                age: 24,
                revision: 2,
            }],
        }
    );

    source.set(SignalList(vec![frank(30)]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 4,
            items: vec![CleanDumpItem {
                id: "2",
                name: "Frank",
                age: 30,
                revision: 3,
            }],
        }
    );

    source.set(SignalList(vec![frank(30)]));
    assert_eq!(
        get_dump(&dump),
        CleanDump {
            list_revision: 4,
            items: vec![CleanDumpItem {
                id: "2",
                name: "Frank",
                age: 30,
                revision: 3,
            }],
        }
    );
}

/// A row notifies only when its own value changes. Rewriting the list with equal
/// content, reordering it, or adding a key must leave the existing rows quiet.
#[test]
fn unchanged_rows_do_not_notify() {
    fn zoe() -> Person {
        Person {
            id: "3",
            name: "Zoe",
            age: 30,
        }
    }

    let source = Value::new(SignalList(vec![bob(), frank(23)]));
    let list = keyed_computed_list(people_computed(&source), |item| item.id);

    let (bob_row, frank_row) = transaction(|ctx| {
        let rows = list.get(ctx);
        (rows[0].value.clone(), rows[1].value.clone())
    });

    let bob_calls = Rc::new(Cell::new(0));
    let frank_calls = Rc::new(Cell::new(0));

    let _bob_sub = bob_row.subscribe({
        let bob_calls = bob_calls.clone();
        move |_| bob_calls.set(bob_calls.get() + 1)
    });
    let _frank_sub = frank_row.subscribe({
        let frank_calls = frank_calls.clone();
        move |_| frank_calls.set(frank_calls.get() + 1)
    });

    // `subscribe` delivers the current value straight away.
    assert_eq!((bob_calls.get(), frank_calls.get()), (1, 1), "initial read");

    // A brand new list carrying structurally equal rows.
    source.set(SignalList(vec![bob(), frank(23)]));
    assert_eq!(
        (bob_calls.get(), frank_calls.get()),
        (1, 1),
        "equal content"
    );

    // Same membership, different order.
    source.set(SignalList(vec![frank(23), bob()]));
    assert_eq!((bob_calls.get(), frank_calls.get()), (1, 1), "reorder");

    // A key appears; the rows that were already there are untouched.
    source.set(SignalList(vec![frank(23), bob(), zoe()]));
    assert_eq!((bob_calls.get(), frank_calls.get()), (1, 1), "new key");

    // Only the row that really changed notifies.
    source.set(SignalList(vec![frank(24), bob(), zoe()]));
    assert_eq!(
        (bob_calls.get(), frank_calls.get()),
        (1, 2),
        "only Frank changed"
    );
}

#[test]
fn keeps_the_first_item_when_duplicate_keys_appear() {
    let source = Value::new(vec![
        Person {
            id: "1",
            name: "first",
            age: 1,
        },
        Person {
            id: "1",
            name: "second",
            age: 2,
        },
        Person {
            id: "2",
            name: "other",
            age: 3,
        },
    ]);

    let list = keyed_computed_list(source.to_computed(), |item| item.id);

    transaction(|ctx| {
        let rows = list.get(ctx);
        let values: Vec<Person> = rows.iter().map(|item| item.value.get(ctx)).collect();
        let keys: Vec<&'static str> = rows.iter().map(|item| item.key).collect();

        assert_eq!(
            values,
            vec![
                Person {
                    id: "1",
                    name: "first",
                    age: 1,
                },
                Person {
                    id: "2",
                    name: "other",
                    age: 3,
                },
            ]
        );
        assert_eq!(keys, vec!["1", "2"]);
    });
}

/// Local copy of the TypeScript `mapKeyedListState` (not public API yet).
fn map_keyed_list_state<T, S, K>(
    list: Computed<Vec<KeyedListItem<K, Computed<T>>>>,
    create_state: impl Fn(Computed<T>) -> S + 'static,
) -> Computed<Vec<KeyedListItem<K, S>>>
where
    T: Clone + 'static,
    S: Clone + 'static,
    K: Clone + Eq + std::hash::Hash + 'static,
{
    let cache = Rc::new(ValueMut::new(HashMap::<K, KeyedListItem<K, S>>::new()));

    Computed::from(move |ctx| {
        let mut result = Vec::new();

        for item in list.get(ctx) {
            let next = cache.change(|cache| {
                if let Some(prev) = cache.get(&item.key) {
                    prev.clone()
                } else {
                    KeyedListItem {
                        key: item.key.clone(),
                        value: create_state(item.value.clone()),
                    }
                }
            });
            result.push(next);
        }

        cache.set(
            result
                .iter()
                .map(|item| (item.key.clone(), item.clone()))
                .collect(),
        );
        result
    })
}

#[test]
fn keyed_list_builds_a_keyed_computed_list() {
    let source = Value::new(vec![Person {
        id: "1",
        name: "Ann",
        age: 20,
    }]);

    let list = keyed_computed_list(source.to_computed(), |item| item.id);

    transaction(|ctx| {
        let rows: Vec<(&'static str, Person)> = list
            .get(ctx)
            .into_iter()
            .map(|item| (item.key, item.value.get(ctx)))
            .collect();

        assert_eq!(
            rows,
            vec![(
                "1",
                Person {
                    id: "1",
                    name: "Ann",
                    age: 20,
                }
            )]
        );
    });
}

#[derive(Clone)]
struct RowState {
    label: Computed<&'static str>,
}

impl PartialEq for RowState {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

#[test]
fn keyed_list_map_runs_create_state_once_per_key() {
    let source = Value::new(vec![Person {
        id: "1",
        name: "Ann",
        age: 20,
    }]);
    let create_count = Rc::new(std::cell::Cell::new(0));

    let list = keyed_computed_list(source.to_computed(), |item| item.id);
    let rows = map_keyed_list_state(list, {
        let create_count = create_count.clone();
        move |person| {
            create_count.set(create_count.get() + 1);
            RowState {
                label: Computed::from({
                    let person = person.clone();
                    move |ctx| person.get(ctx).name
                }),
            }
        }
    });

    let first_label_id = transaction(|ctx| {
        let rows = rows.get(ctx);
        assert_eq!(create_count.get(), 1);
        assert_eq!(rows[0].value.label.get(ctx), "Ann");
        rows[0].value.label.id()
    });

    source.set(vec![Person {
        id: "1",
        name: "Ann",
        age: 21,
    }]);

    transaction(|ctx| {
        let current = rows.get(ctx);
        assert_eq!(create_count.get(), 1);
        assert_eq!(current[0].value.label.id(), first_label_id);
        assert_eq!(current[0].value.label.get(ctx), "Ann");
    });

    source.set(vec![
        Person {
            id: "1",
            name: "Ann",
            age: 21,
        },
        Person {
            id: "2",
            name: "Bob",
            age: 30,
        },
    ]);

    transaction(|ctx| {
        let current = rows.get(ctx);
        assert_eq!(current.len(), 2);
        assert_eq!(create_count.get(), 2);
        assert_eq!(current[0].value.label.id(), first_label_id);
    });
}

#[test]
fn returns_last_value_after_key_leaves_the_list() {
    let source = Value::new(vec![bob()]);
    let list = keyed_computed_list(source.to_computed(), |item| item.id);

    let stale_item = transaction(|ctx| list.get(ctx)[0].value.clone());

    source.set(Vec::new());

    transaction(|ctx| {
        assert_eq!(list.get(ctx).len(), 0);
        assert_eq!(stale_item.get(ctx), bob());
    });
}
