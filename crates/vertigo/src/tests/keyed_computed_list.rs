use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    Computed, DropResource, KeyedListItem, Value, keyed_computed_list, struct_mut::ValueMut,
    transaction,
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

/// Building a keyed list allocates graph nodes. Doing that from a computed that re-runs
/// while the graph is being refreshed must work - a render closure reached during an update
/// is exactly that situation.
#[test]
fn can_be_built_during_a_refresh() {
    let trigger = Value::new(1);

    let ages = Computed::from({
        let trigger = trigger.clone();
        move |ctx| {
            let age = trigger.get(ctx);
            let source = Value::new(vec![Person {
                id: "1",
                name: "Ann",
                age,
            }]);

            let list = keyed_computed_list(source.to_computed(), |item| item.id);

            list.get(ctx)
                .iter()
                .map(|row| row.value.get(ctx).age)
                .collect::<Vec<_>>()
        }
    });

    let seen = Rc::new(RefCell::new(Vec::new()));
    let _subscription = ages.subscribe({
        let seen = seen.clone();
        move |ages| seen.borrow_mut().push(ages)
    });

    assert_eq!(*seen.borrow(), vec![vec![1]]);

    // Re-runs the closure - and so builds a second keyed list - mid-refresh.
    trigger.set(2);

    assert_eq!(*seen.borrow(), vec![vec![1], vec![2]]);
}

/// The duplicate-key path logs and skips; make sure it does so safely mid-refresh too.
#[test]
fn duplicate_keys_during_a_refresh() {
    let trigger = Value::new(1);

    let names = Computed::from({
        let trigger = trigger.clone();
        move |ctx| {
            let age = trigger.get(ctx);
            let source = Value::new(vec![
                Person {
                    id: "1",
                    name: "first",
                    age,
                },
                Person {
                    id: "1",
                    name: "duplicate",
                    age,
                },
            ]);

            let list = keyed_computed_list(source.to_computed(), |item| item.id);

            list.get(ctx)
                .iter()
                .map(|row| row.value.get(ctx).name)
                .collect::<Vec<_>>()
        }
    });

    let seen = Rc::new(RefCell::new(Vec::new()));
    let _subscription = names.subscribe({
        let seen = seen.clone();
        move |names| seen.borrow_mut().push(names)
    });

    trigger.set(2);

    assert_eq!(*seen.borrow(), vec![vec!["first"]]);
}

/// Counts how often the item type is cloned, to pin down the cost of an update.
#[derive(Debug)]
struct Counted {
    id: u32,
    value: u32,
    clones: Rc<Cell<usize>>,
}

impl Clone for Counted {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);

        Counted {
            id: self.id,
            value: self.value,
            clones: self.clones.clone(),
        }
    }
}

impl PartialEq for Counted {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.value == other.value
    }
}

/// Build a list of `rows` rows, observe every row the way a rendered list does, then
/// change a single row and report how many item clones that update cost.
fn clones_for_one_row_update(rows: u32) -> usize {
    let clones = Rc::new(Cell::new(0));

    let build = |first_value: u32| {
        (0..rows)
            .map(|id| Counted {
                id,
                value: if id == 0 { first_value } else { id },
                clones: clones.clone(),
            })
            .collect::<Vec<_>>()
    };

    let source = Value::new(build(0));
    let list = keyed_computed_list(source.to_computed(), |item| item.id);

    // Observe the same way a rendered list does: the list itself, plus every row.
    let _row_subscriptions = transaction(|ctx| list.get(ctx))
        .into_iter()
        .map(|item| item.value.subscribe(|_| {}))
        .collect::<Vec<_>>();
    let _list_subscription = list.subscribe(|_| {});

    clones.set(0);
    source.set(build(1));
    clones.get()
}

/// Changing one row must cost work proportional to the list, not to its square.
///
/// Every row's `Computed` reads the shared key->value map, and `Computed::get` hands
/// back a clone of the cached value - so if that map is not behind an `Rc`, each of
/// the n rows copies all n items on every update.
#[test]
fn one_row_update_scales_linearly() {
    let small = clones_for_one_row_update(20);
    let large = clones_for_one_row_update(80);

    assert!(
        large < small * 6,
        "updating one row looks quadratic: 20 rows cost {small} clones, \
         80 rows cost {large} (linear would be about 4x, quadratic about 16x)"
    );
}

/// A row that outlives its key keeps its own last value, and nothing else.
///
/// Rows share one `Rc` to the key-value map while they are live, which is what keeps a read
/// down to a single item clone. A row whose key has left cannot share it any more - the map
/// has moved on - so it has to let go of the map rather than pin every item that was in the
/// list when it was last seen. The counts below are for the *same* stale row in a ten-row and
/// a hundred-row list: if they differ, the row is holding the list.
#[test]
fn a_stale_row_does_not_pin_the_rest_of_the_list() {
    #[derive(Clone, Debug)]
    struct Tracked {
        id: u32,
        /// Never read - counted. `Rc::strong_count` on the handle below is how many copies
        /// of this item are still alive somewhere in the graph.
        #[expect(dead_code, reason = "counted through Rc::strong_count, not read")]
        alive: Rc<()>,
    }

    impl PartialEq for Tracked {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    let retained_after_clearing = |rows: u32| {
        let alive = Rc::new(());

        let source = Value::new(
            (0..rows)
                .map(|id| Tracked {
                    id,
                    alive: alive.clone(),
                })
                .collect::<Vec<_>>(),
        );
        let list = keyed_computed_list(source.to_computed(), |item| item.id);
        let stale = transaction(|ctx| list.get(ctx)[3].value.clone());

        source.set(Vec::new());

        // Reading a departed key is what tells the row its key is gone.
        transaction(|ctx| {
            assert_eq!(stale.get(ctx).id, 3);
        });

        Rc::strong_count(&alive)
    };

    let small = retained_after_clearing(10);
    let large = retained_after_clearing(100);

    assert_eq!(
        small, large,
        "a stale row retained {small} items out of 10 and {large} out of 100, \
         so it is holding on to the list rather than to its own value"
    );
}

/// Counts what a keyed list actually does to its keys: how often it hashes one, and how
/// often it copies one.
#[derive(Debug, Default)]
struct KeyWork {
    hashes: Cell<usize>,
    clones: Cell<usize>,
}

/// A key that reports itself. Identity and hashing both come from `id` alone, so the
/// counters change nothing about how the list behaves.
#[derive(Debug)]
struct CountedKey {
    id: u32,
    work: Rc<KeyWork>,
}

impl Clone for CountedKey {
    fn clone(&self) -> Self {
        self.work.clones.set(self.work.clones.get() + 1);

        CountedKey {
            id: self.id,
            work: self.work.clone(),
        }
    }
}

impl std::hash::Hash for CountedKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.work.hashes.set(self.work.hashes.get() + 1);
        self.id.hash(state);
    }
}

impl PartialEq for CountedKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CountedKey {}

#[derive(Clone, Debug)]
struct KeyedRow {
    key: CountedKey,
    value: u32,
}

impl PartialEq for KeyedRow {
    fn eq(&self, other: &Self) -> bool {
        self.key.id == other.key.id && self.value == other.value
    }
}

/// Build `rows` rows, observe them the way a rendered list does, change one row's value, and
/// report the key work that single update cost.
fn key_work_for_one_row_update(rows: u32) -> (usize, usize) {
    let work = Rc::new(KeyWork::default());

    let build = |first_value: u32| {
        (0..rows)
            .map(|id| KeyedRow {
                key: CountedKey {
                    id,
                    work: work.clone(),
                },
                value: if id == 0 { first_value } else { id },
            })
            .collect::<Vec<_>>()
    };

    let source = Value::new(build(0));
    let list = keyed_computed_list(source.to_computed(), |item| item.key.clone());

    let _row_subscriptions = transaction(|ctx| list.get(ctx))
        .into_iter()
        .map(|item| item.value.subscribe(|_| {}))
        .collect::<Vec<_>>();
    let _list_subscription = list.subscribe(|_| {});

    work.hashes.set(0);
    work.clones.set(0);
    source.set(build(1));

    (work.hashes.get(), work.clones.get())
}

/// A budget, not a shape.
///
/// [`one_row_update_scales_linearly`] pins the exponent; this pins the coefficient, which is
/// the thing an ordinary-looking edit here regresses. Every count below is per row of a list
/// that is only being *read* - one row's value changed, membership and order did not - so
/// each unit of it is paid by every row of every list on every update.
///
/// Measured at 100 rows: 302 hashes and 601 key clones, against 919 and 1201 for the version
/// that rebuilt three indexes and two caches on every pass. The budgets below sit above those
/// figures with room for a small change and well under room for a doubling. If this fails,
/// something started rebuilding an index again.
#[test]
fn one_row_update_stays_within_its_key_budget() {
    const ROWS: u32 = 100;
    const HASHES_PER_ROW: usize = 4;
    const CLONES_PER_ROW: usize = 7;

    let (hashes, clones) = key_work_for_one_row_update(ROWS);

    assert!(
        hashes <= ROWS as usize * HASHES_PER_ROW,
        "one update of {ROWS} rows hashed a key {hashes} times, budget is {}",
        ROWS as usize * HASHES_PER_ROW
    );
    assert!(
        clones <= ROWS as usize * CLONES_PER_ROW,
        "one update of {ROWS} rows cloned a key {clones} times, budget is {}",
        ROWS as usize * CLONES_PER_ROW
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
    T: Clone + PartialEq + 'static,
    S: Clone + PartialEq + 'static,
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
