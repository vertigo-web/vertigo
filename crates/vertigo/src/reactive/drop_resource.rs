use std::{any::Any, cell::Cell};

thread_local! {
    static DROP_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// `true` while a [`DropResource`] destructor is running. `Value::set` is refused then:
/// drop only tears down external subscriptions, it does not write back into the graph.
pub(crate) fn in_drop() -> bool {
    DROP_DEPTH.with(|d| d.get() > 0)
}

/// Clears the drop mark when the destructor returns (including panic).
struct Dropping;

impl Drop for Dropping {
    fn drop(&mut self) {
        DROP_DEPTH.with(|d| d.set(d.get() - 1));
    }
}

/// Runs a destructor when dropped (or holds a value whose `Drop` cleans up).
pub enum DropResource {
    Fun(Option<Box<dyn FnOnce()>>),
    Struct(Box<dyn Any>),
}

impl DropResource {
    pub fn new<F: FnOnce() + 'static>(drop_fun: F) -> DropResource {
        DropResource::Fun(Some(Box::new(drop_fun)))
    }

    pub fn from_struct(inst: impl Any) -> DropResource {
        DropResource::Struct(Box::new(inst))
    }

    pub fn off(self) {}
}

impl PartialEq for DropResource {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Drop for DropResource {
    fn drop(&mut self) {
        match self {
            Self::Fun(inner) => {
                let drop_fun = std::mem::take(inner);

                if let Some(drop_fun) = drop_fun {
                    DROP_DEPTH.with(|d| d.set(d.get() + 1));
                    let _guard = Dropping;
                    drop_fun();
                }
            }
            Self::Struct(_) => {}
        }
    }
}
