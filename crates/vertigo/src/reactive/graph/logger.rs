#[cfg(test)]
use std::cell::Cell;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

struct LoggerInner {
    #[cfg(test)]
    next_id: Cell<u64>,
    buffers: RefCell<BTreeMap<u64, Rc<RefCell<Vec<String>>>>>,
}

/// Where a graph reports the rules it had to enforce (a refused write, a connect loop).
///
/// Every message goes to `log::error!`. Listening to them is a test facility and is not
/// part of the public API: an application observes them through its `log` implementation,
/// which is where a buffer that nobody drains would be its own problem.
#[derive(Clone)]
pub(crate) struct Logger {
    inner: Rc<LoggerInner>,
}

/// One subscription to a [`Logger`]. [`Drop`] unregisters it.
#[cfg(test)]
pub(crate) struct LoggerListener {
    logger: Rc<LoggerInner>,
    id: u64,
    buffer: Rc<RefCell<Vec<String>>>,
}

impl Logger {
    pub(super) fn new() -> Self {
        Logger {
            inner: Rc::new(LoggerInner {
                #[cfg(test)]
                next_id: Cell::new(1),
                buffers: RefCell::new(BTreeMap::new()),
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn listen(&self) -> LoggerListener {
        let id = self.inner.next_id.get();
        self.inner.next_id.set(id + 1);
        let buffer = Rc::new(RefCell::new(Vec::new()));
        self.inner.buffers.borrow_mut().insert(id, buffer.clone());
        LoggerListener {
            logger: self.inner.clone(),
            id,
            buffer,
        }
    }

    pub(super) fn error(&self, message: &str) {
        log::error!("{message}");
        for buffer in self.inner.buffers.borrow().values() {
            buffer.borrow_mut().push(message.to_string());
        }
    }
}

#[cfg(test)]
impl LoggerListener {
    /// Current messages, then empty the buffer.
    pub(crate) fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.buffer.borrow_mut())
    }

    /// [`take`](Self::take) and compare with `expected`. Panic points at the caller.
    #[track_caller]
    pub(crate) fn assert_eq(&self, expected: &[&str]) {
        assert_eq!(self.take(), expected);
    }
}

#[cfg(test)]
impl Drop for LoggerListener {
    fn drop(&mut self) {
        self.logger.buffers.borrow_mut().remove(&self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_returns_messages_and_clears() {
        let logger = Logger::new();
        let listener = logger.listen();
        logger.error("a");
        logger.error("b");
        assert_eq!(listener.take(), ["a", "b"]);
        listener.assert_eq(&[]);
        logger.error("c");
        listener.assert_eq(&["c"]);
    }

    #[test]
    fn drop_unregisters() {
        let logger = Logger::new();
        let listener = logger.listen();
        drop(listener);
        logger.error("x");
        assert!(logger.inner.buffers.borrow().is_empty());
    }

    #[test]
    fn two_listeners_both_receive() {
        let logger = Logger::new();
        let a = logger.listen();
        let b = logger.listen();
        logger.error("x");
        assert_eq!(a.take(), ["x"]);
        assert_eq!(b.take(), ["x"]);
    }
}
