use std::cell::Cell;

/// Nesting depth of `Graph::transaction` and the reentrancy flag for `propagate`.
pub(super) struct Transaction {
    depth: Cell<u32>,
    propagating: Cell<bool>,
}

/// The outermost transaction just closed.
pub(super) struct OuterLeave {
    /// A write landed while `propagate` was already running (skip nested hooks).
    pub already_propagating: bool,
}

/// Clears `propagating` when the wave ends (including panic).
pub(super) struct Propagating<'a> {
    tx: &'a Transaction,
}

impl Drop for Propagating<'_> {
    fn drop(&mut self) {
        self.tx.propagating.set(false);
    }
}

impl Transaction {
    pub(super) fn new() -> Self {
        Self {
            depth: Cell::new(0),
            propagating: Cell::new(false),
        }
    }

    pub(super) fn enter(&self) {
        self.depth.set(self.depth.get() + 1);
    }

    /// Close one nesting level. `Some` when this was the outermost transaction.
    pub(super) fn leave(&self) -> Option<OuterLeave> {
        let depth = self.depth.get();
        debug_assert!(depth > 0);
        self.depth.set(depth - 1);
        if depth == 1 {
            Some(OuterLeave {
                already_propagating: self.propagating.get(),
            })
        } else {
            None
        }
    }

    /// `true` when no transaction is open and no propagate wave is running.
    pub(super) fn can_propagate(&self) -> bool {
        self.depth.get() == 0 && !self.propagating.get()
    }

    /// Start a propagate wave. Call only when [`Self::can_propagate`] is `true`.
    pub(super) fn start_propagate(&self) -> Propagating<'_> {
        debug_assert!(self.can_propagate());
        self.propagating.set(true);
        Propagating { tx: self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_propagate_while_propagating() {
        let tx = Transaction::new();
        let _wave = tx.start_propagate();
        assert!(!tx.can_propagate());
    }
}
