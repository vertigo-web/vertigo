//! `#[store(all = name)]` on a real store. Each test runs in a thread of its own, so with a
//! store of its own too.

use std::{cell::Cell, rc::Rc};

use vertigo::store;

thread_local! {
    static CREATED: Cell<u32> = const { Cell::new(0) };
}

#[store(all = all_counters)]
fn counter(name: &String, start: &u32) -> Rc<Cell<u32>> {
    CREATED.with(|created| created.set(created.get() + 1));
    Rc::new(Cell::new(*start))
}

fn counter_of(name: &str, start: u32) -> u32 {
    counter(&name.to_string(), &start).get()
}

#[test]
fn all_gives_every_value_made() {
    assert!(all_counters().is_empty());

    counter_of("a", 1);
    counter_of("b", 2);
    counter_of("a", 1);
    counter_of("a", 3);

    let mut values = all_counters()
        .iter()
        .map(|value| value.get())
        .collect::<Vec<_>>();
    values.sort();
    assert_eq!(values, [1, 2, 3]);
    assert_eq!(CREATED.with(Cell::get), 3);
}

#[test]
fn all_gives_the_values_the_store_hands_out() {
    counter_of("a", 1);
    counter_of("b", 2);

    for value in all_counters() {
        value.set(0);
        // what is done with them may use the store again
        counter_of("c", 3);
    }

    assert_eq!(counter_of("a", 1), 0);
    assert_eq!(counter_of("b", 2), 0);
    assert_eq!(counter_of("c", 3), 3);
    assert_eq!(CREATED.with(Cell::get), 3);
}
