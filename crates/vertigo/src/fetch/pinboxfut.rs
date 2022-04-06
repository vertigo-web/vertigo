use std::pin::Pin;
use std::future::Future;

pub type PinBox<T> = Pin<Box<T>>;

pub type PinBoxFuture<T> = PinBox<dyn Future<Output=T> + 'static>;
