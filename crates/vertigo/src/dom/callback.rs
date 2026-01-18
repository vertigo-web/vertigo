use std::rc::Rc;

use vertigo_macro::bind;

use crate::computed::{Computed, DropResource, struct_mut::ValueMut};

pub enum Callback<R> {
    Basic(Rc<dyn Fn() -> R + 'static>),
    Computed(Computed<Rc<dyn Fn() -> R + 'static>>),
}

impl<R> From<Rc<dyn Fn() -> R + 'static>> for Callback<R> {
    fn from(value: Rc<dyn Fn() -> R + 'static>) -> Self {
        Callback::Basic(value)
    }
}

impl<R, F: Fn() -> R + 'static> From<F> for Callback<R> {
    fn from(value: F) -> Self {
        Callback::Basic(Rc::new(value))
    }
}

impl<R> From<Computed<Rc<dyn Fn() -> R + 'static>>> for Callback<R> {
    fn from(value: Computed<Rc<dyn Fn() -> R + 'static>>) -> Self {
        Callback::Computed(value)
    }
}

impl<R: 'static> Callback<R> {
    pub fn subscribe(&self) -> (Rc<dyn Fn() -> R + 'static>, Option<DropResource>) {
        match self {
            Self::Basic(func) => (func.clone(), None),
            Self::Computed(computed) => {
                let current = Rc::new(ValueMut::new(None));

                let drop = computed.clone().subscribe_all(bind!(current, |new_fn| {
                    current.set(Some(new_fn));
                }));

                let callback = Rc::new(move || -> R {
                    let callback = current.get();

                    let Some(callback) = callback else {
                        unreachable!();
                    };

                    callback()
                });

                (callback, Some(drop))
            }
        }
    }
}

pub enum Callback1<T, R> {
    Basic(Rc<dyn Fn(T) -> R + 'static>),
    Rc(Rc<dyn Fn(T) -> R + 'static>),
    Computed(Computed<Rc<dyn Fn(T) -> R + 'static>>),
}

impl<T, R, F: Fn(T) -> R + 'static> From<F> for Callback1<T, R> {
    fn from(value: F) -> Self {
        Callback1::Basic(Rc::new(value))
    }
}

impl<T, R> From<Rc<dyn Fn(T) -> R + 'static>> for Callback1<T, R> {
    fn from(value: Rc<dyn Fn(T) -> R + 'static>) -> Self {
        Callback1::Rc(value)
    }
}

impl<T, R> From<Computed<Rc<dyn Fn(T) -> R + 'static>>> for Callback1<T, R> {
    fn from(value: Computed<Rc<dyn Fn(T) -> R + 'static>>) -> Self {
        Callback1::Computed(value)
    }
}

impl<T: 'static, R: 'static> Callback1<T, R> {
    pub fn subscribe(&self) -> (Rc<dyn Fn(T) -> R + 'static>, Option<DropResource>) {
        match self {
            Self::Basic(func) => (func.clone(), None),
            Self::Rc(func) => (func.clone(), None),
            Self::Computed(computed) => {
                let current = Rc::new(ValueMut::new(None));

                let drop = computed.clone().subscribe_all(bind!(current, |new_fn| {
                    current.set(Some(new_fn));
                }));

                let callback = Rc::new(move |param: T| -> R {
                    let callback = current.get();

                    let Some(callback) = callback else {
                        unreachable!();
                    };

                    callback(param)
                });

                (callback, Some(drop))
            }
        }
    }
}
