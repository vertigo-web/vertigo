
use std::rc::Rc;

use super::{BoxRefCell, EqBox};

type CallbackType<T> = Rc<dyn Fn(T) -> ()>;
type CallbackBox<T> = EqBox<Rc<BoxRefCell<Option<CallbackType<T>>>>>;

pub struct Action<T: 'static> {
    callback: CallbackBox<T>,
}

impl<T: 'static> Clone for Action<T> {
    fn clone(&self) -> Self {
        Action {
            callback: self.callback.clone(),
        }
    }
}

impl<T: 'static> Action<T> {
    pub fn trigger(&self, message: T) {
        let Action { callback } = self;

        let callback_fn = callback.get(|state| {
            state.clone()
        });

        if let Some(callback_fn) = callback_fn {
            callback_fn(message);
        } else {
            log::error!("Action not launched");
        }
    }

    pub fn new() -> (Action<T>, ActionSubscribe<T>) {
        let callback = EqBox::new(Rc::new(BoxRefCell::new(None, "action")));

        let subscribe = ActionSubscribe::new(callback.clone());
        let action = Action {
            callback
        };

        (action, subscribe)
    }
}

impl<T: 'static> PartialEq for Action<T> {
    fn eq(&self, other: &Action<T>) -> bool {
        self.callback == other.callback
    }
}

pub struct ActionSubscribe<T: 'static> {
    callback: CallbackBox<T>,
}

impl<T: 'static> ActionSubscribe<T> {
    pub fn new(callback: CallbackBox<T>) -> ActionSubscribe<T> {
        ActionSubscribe {
            callback
        }
    }

    pub fn subscribe<F: Fn(T) -> () + 'static>(self, fun: F) {
        let callback = self.callback.clone();
        callback.change(Rc::new(fun), |state, data| {
            *state = Some(data);
        });
    }
}

impl<T: 'static> Drop for ActionSubscribe<T> {
    fn drop(&mut self) {
        let ActionSubscribe { callback } = self;

        let is_some = callback.get(|state| {
            state.is_some()
        });

        if !is_some {
            log::error!("Not subscribed");
        }
    }
}
