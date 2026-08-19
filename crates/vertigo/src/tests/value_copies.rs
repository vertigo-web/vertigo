//! What a `Value` write costs when nothing is listening to it.

use std::{cell::Cell, rc::Rc};

use crate::Value;

/// Counts how often an item is cloned.
#[derive(Debug)]
struct Counted {
    value: u32,
    clones: Rc<Cell<usize>>,
}

impl Clone for Counted {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);

        Counted {
            value: self.value,
            clones: self.clones.clone(),
        }
    }
}

impl PartialEq for Counted {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

fn list(len: u32, offset: u32, clones: &Rc<Cell<usize>>) -> Vec<Counted> {
    (0..len)
        .map(|value| Counted {
            value: value + offset,
            clones: clones.clone(),
        })
        .collect()
}

#[test]
fn new_takes_ownership_of_the_payload() {
    let clones = Rc::new(Cell::new(0));

    let _value = Value::new(list(10, 0, &clones));

    assert_eq!(clones.get(), 0, "`Value::new` must not copy what it stores");
}

#[test]
fn set_without_listeners_does_not_copy_the_payload() {
    let clones = Rc::new(Cell::new(0));
    let value = Value::new(list(10, 0, &clones));

    clones.set(0);
    value.set(list(10, 100, &clones));

    assert_eq!(clones.get(), 0, "a write nobody observes must not copy");
}

#[test]
fn set_still_delivers_to_listeners() {
    let clones = Rc::new(Cell::new(0));
    let value = Value::new(list(3, 0, &clones));

    let seen = Rc::new(Cell::new(0));
    let _event = value.add_event({
        let seen = seen.clone();
        move |list: Vec<Counted>| seen.set(list.len())
    });

    value.set(list(3, 100, &clones));

    assert_eq!(seen.get(), 3);
}
