use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    computed::{DropResource, struct_mut::ValueMut},
    dev::{CallbackId, command::TimerKind},
};

use super::{CallbackStore, api_browser_command};

#[store]
pub fn api_timers() -> Rc<ApiTimers> {
    ApiTimers::new()
}

#[cfg(test)]
type MockTimerHandler = crate::dev::ValueMut<Option<Rc<dyn Fn(u32, CallbackId, TimerKind)>>>;

pub struct ApiTimers {
    timers: CallbackStore<(), ()>,
    #[cfg(test)]
    mock_handler: MockTimerHandler,
}

impl ApiTimers {
    fn new() -> Rc<ApiTimers> {
        Rc::new(ApiTimers {
            timers: CallbackStore::new(),
            #[cfg(test)]
            mock_handler: crate::dev::ValueMut::new(None),
        })
    }

    #[cfg(test)]
    pub fn set_mock_handler(&self, handler: impl Fn(u32, CallbackId, TimerKind) + 'static) {
        self.mock_handler.set(Some(Rc::new(handler)));
    }

    fn set<F: Fn() + 'static>(&self, duration: u32, callback: F, kind: TimerKind) -> DropResource {
        let (callback_id, drop) = self.timers.register(move |_| {
            callback();
        });

        #[cfg(test)]
        if let Some(handler) = self.mock_handler.get() {
            handler(duration, callback_id, kind);
            return DropResource::new(move || {
                drop.off();
            });
        }

        api_browser_command().timer_set(callback_id, duration, kind);

        DropResource::new(move || {
            api_browser_command().timer_clear(callback_id);
            drop.off();
        })
    }

    pub fn timeout<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        self.set(duration, callback, TimerKind::Timeout)
    }

    pub fn interval<F: Fn() + 'static>(&self, duration: u32, callback: F) -> DropResource {
        self.set(duration, callback, TimerKind::Interval)
    }

    pub fn callback_timeout(&self, callback: CallbackId) {
        self.timers.call(callback, ());
    }

    pub fn set_timeout_and_detach<F: Fn() + 'static>(&self, duration: u32, callback: F) {
        let drop_box: Rc<ValueMut<Option<DropResource>>> = Rc::new(ValueMut::new(None));

        let callback_with_drop = {
            let drop_box = drop_box.clone();

            move || {
                callback();
                drop_box.set(None);
            }
        };

        let drop = self.timeout(duration, callback_with_drop);
        drop_box.set(Some(drop));
    }
}
