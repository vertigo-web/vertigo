use std::cell::Cell;

/// Nesting depth of `Graph::transaction`, the reentrancy flag for `propagate`,
/// and the depth of graph callbacks (`compute` / `subscribe`) that must not write.
pub(super) struct Transaction {
    depth: Cell<u32>,
    propagating: Cell<bool>,
    callback_depth: Cell<u32>,
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

/// Restores the transaction depth if the transaction body unwinds.
///
/// The normal path goes through [`Self::leave`], which also reports whether this was the
/// outermost level so the caller can run the wave. A panic has nobody to report to, and
/// propagating over half-applied state would be worse than not propagating at all - so an
/// unwind only restores the depth. Without this, one panic inside a transaction would leave
/// the depth above zero forever and the graph would never propagate again.
pub(super) struct DepthGuard<'a> {
    tx: &'a Transaction,
    armed: bool,
}

impl DepthGuard<'_> {
    /// Close the level normally. `Some` when this was the outermost transaction.
    pub(super) fn leave(mut self) -> Option<OuterLeave> {
        self.armed = false;
        self.tx.leave()
    }
}

impl Drop for DepthGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.tx.leave();
        }
    }
}

/// Decrements callback depth when a `compute` / `subscribe` closure returns (including panic).
pub(crate) struct CallbackGuard<'a> {
    tx: &'a Transaction,
}

impl Drop for CallbackGuard<'_> {
    fn drop(&mut self) {
        self.tx.callback_depth.set(self.tx.callback_depth.get() - 1);
    }
}

impl Transaction {
    pub(super) fn new() -> Self {
        Self {
            depth: Cell::new(0),
            propagating: Cell::new(false),
            callback_depth: Cell::new(0),
        }
    }

    /// Increment nesting. `true` when this opened the outermost transaction.
    ///
    /// The guard restores the depth if the transaction body unwinds; close the level normally
    /// with [`DepthGuard::leave`].
    pub(super) fn enter(&self) -> (bool, DepthGuard<'_>) {
        let depth = self.depth.get();
        self.depth.set(depth + 1);
        (
            depth == 0,
            DepthGuard {
                tx: self,
                armed: true,
            },
        )
    }

    /// Close one nesting level. `Some` when this was the outermost transaction.
    fn leave(&self) -> Option<OuterLeave> {
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

    pub(super) fn is_propagating(&self) -> bool {
        self.propagating.get()
    }

    /// Start a propagate wave. Call only when [`Self::can_propagate`] is `true`.
    pub(super) fn start_propagate(&self) -> Propagating<'_> {
        debug_assert!(self.can_propagate());
        self.propagating.set(true);
        Propagating { tx: self }
    }

    pub(super) fn enter_callback(&self) -> CallbackGuard<'_> {
        self.callback_depth.set(self.callback_depth.get() + 1);
        CallbackGuard { tx: self }
    }

    /// `Value::set` is forbidden from `compute` / `subscribe` and while a wave is running.
    pub(super) fn writes_blocked(&self) -> bool {
        self.callback_depth.get() > 0 || self.propagating.get()
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
        assert!(tx.writes_blocked());
    }

    #[test]
    fn writes_blocked_inside_callback() {
        let tx = Transaction::new();
        assert!(!tx.writes_blocked());
        let _guard = tx.enter_callback();
        assert!(tx.writes_blocked());
    }
}
