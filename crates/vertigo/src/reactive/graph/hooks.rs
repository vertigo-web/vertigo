use std::{cell::Cell, rc::Rc};

use crate::struct_mut::ValueMut;

/// One registered callback, paired with the id `remove` takes back.
type Hook = (u64, Rc<dyn Fn()>);

/// Callbacks fired after a completed transaction (once `propagate` has finished,
/// before `when_connect` / disconnect).
pub(super) struct Hooks {
    next_id: Cell<u64>,
    hooks: ValueMut<Vec<Hook>>,
}

impl Hooks {
    pub(super) fn new() -> Self {
        Self {
            next_id: Cell::new(1),
            hooks: ValueMut::new(Vec::new()),
        }
    }

    pub(super) fn insert(&self, callback: impl Fn() + 'static) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        self.hooks
            .change(|hooks| hooks.push((id, Rc::new(callback))));
        id
    }

    pub(super) fn remove(&self, id: u64) {
        self.hooks
            .change(|hooks| hooks.retain(|(hook_id, _)| *hook_id != id));
    }

    pub(super) fn fire(&self) {
        if self.hooks.map(|hooks| hooks.is_empty()) {
            return;
        }
        let hooks: Vec<_> = self
            .hooks
            .map(|hooks| hooks.iter().map(|(_, hook)| hook.clone()).collect());
        for hook in hooks {
            hook();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    /// A callback can hold what removes another hook when it goes.
    #[test]
    fn remove_drops_a_hook_that_removes_another() {
        struct Removes(std::rc::Weak<Hooks>, u64);
        impl Drop for Removes {
            fn drop(&mut self) {
                if let Some(hooks) = self.0.upgrade() {
                    hooks.remove(self.1);
                }
            }
        }

        let hooks = Rc::new(Hooks::new());
        let other = hooks.insert(|| {});
        let removes = Removes(Rc::downgrade(&hooks), other);
        let first = hooks.insert(move || {
            let _ = &removes;
        });

        hooks.remove(first);

        assert!(
            hooks.hooks.map(|hooks| hooks.is_empty()),
            "the other hook is gone too"
        );
    }

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
