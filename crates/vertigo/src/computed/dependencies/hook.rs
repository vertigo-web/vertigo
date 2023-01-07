use crate::{driver_module::event_emmiter::EventEmmiter, DropResource};

#[derive(Clone)]
pub struct Hooks {
    callback_after_transaction: EventEmmiter<()>,
}

impl Hooks {
    pub fn new() -> Hooks {
        Hooks {
            callback_after_transaction: EventEmmiter::default(),
        }
    }

    #[must_use]
    pub(crate) fn on_after_transaction(&self, callback: impl Fn() + 'static) -> DropResource {
        self.callback_after_transaction.add(move |_| {
            callback();
        })
    }

    pub fn fire_end(&self) {
        self.callback_after_transaction.trigger(());
    }
}
