use std::rc::Rc;

pub enum Callback<R: 'static> {
    Basic(Rc<dyn Fn() -> R + 'static>),
}

impl<R: 'static> From<Rc<dyn Fn() -> R + 'static>> for Callback<R> {
    fn from(value: Rc<dyn Fn() -> R + 'static>) -> Self {
        Callback::Basic(value)
    }
}

impl<R: 'static, F: Fn() -> R + 'static> From<F> for Callback<R> {
    fn from(value: F) -> Self {
        Callback::Basic(Rc::new(value))
    }
}

impl<R: 'static> Callback<R> {
    pub fn subscribe(&self) -> (Rc<dyn Fn() -> R + 'static>, Option<crate::DropResource>) {
        match self {
            Self::Basic(func) => (func.clone(), None),
        }
    }
}

pub enum Callback1<T: 'static, R: 'static> {
    Basic(Rc<dyn Fn(T) -> R + 'static>),
    Rc(Rc<dyn Fn(T) -> R + 'static>),
}

impl<T: 'static, R: 'static, F: Fn(T) -> R + 'static> From<F> for Callback1<T, R> {
    fn from(value: F) -> Self {
        Callback1::Basic(Rc::new(value))
    }
}

impl<T: 'static, R: 'static> From<Rc<dyn Fn(T) -> R + 'static>> for Callback1<T, R> {
    fn from(value: Rc<dyn Fn(T) -> R + 'static>) -> Self {
        Callback1::Rc(value)
    }
}

impl<T: 'static, R: 'static> Callback1<T, R> {
    pub fn subscribe(&self) -> (Rc<dyn Fn(T) -> R + 'static>, Option<crate::DropResource>) {
        match self {
            Self::Basic(func) => (func.clone(), None),
            Self::Rc(func) => (func.clone(), None),
        }
    }
}
