use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

/// Callbacks fired after a completed transaction (once `propagate` has finished,
/// before `when_connect` / disconnect).
pub(super) struct Hooks {
    next_id: Cell<u64>,
    hooks: RefCell<BTreeMap<u64, Rc<dyn Fn()>>>,
}

impl Hooks {
    pub(super) fn new() -> Self {
        Self {
            next_id: Cell::new(1),
            hooks: RefCell::new(BTreeMap::new()),
        }
    }

    pub(super) fn insert(&self, callback: impl Fn() + 'static) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        self.hooks.borrow_mut().insert(id, Rc::new(callback));
        id
    }

    pub(super) fn remove(&self, id: u64) {
        self.hooks.borrow_mut().remove(&id);
    }

    pub(super) fn fire(&self) {
        if self.hooks.borrow().is_empty() {
            return;
        }
        let hooks: Vec<_> = self.hooks.borrow().values().cloned().collect();
        for hook in hooks {
            hook();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn remove_does_not_fire() {
        let hooks = Hooks::new();
        let n = Rc::new(Cell::new(0));
        let id = hooks.insert({
            let n = n.clone();
            move || n.set(1)
        });
        hooks.remove(id);
        hooks.fire();
        assert_eq!(n.get(), 0);
    }

    #[test]
    fn two_hooks_both_fire() {
        let hooks = Hooks::new();
        let a = Rc::new(Cell::new(0));
        let b = Rc::new(Cell::new(0));
        hooks.insert({
            let a = a.clone();
            move || a.set(1)
        });
        hooks.insert({
            let b = b.clone();
            move || b.set(1)
        });
        hooks.fire();
        assert_eq!(a.get(), 1);
        assert_eq!(b.get(), 1);
    }
}
