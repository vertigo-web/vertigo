use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

// https://users.rust-lang.org/t/how-to-receive-a-async-callback/40110/3
// https://users.rust-lang.org/t/can-you-turn-a-callback-into-a-future-into-async-await/49378/16
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=10cd4b012f678705c9d78fed5ca81857

#[derive(Default)]
pub struct CallbackFutureInner<T> {
    waker: Cell<Option<Waker>>,
    result: Cell<Option<T>>,
}

impl<T> CallbackFutureInner<T> {
    fn new() -> Rc<CallbackFutureInner<T>> {
        Rc::new(CallbackFutureInner {
            waker: Cell::new(None),
            result: Cell::new(None),
        })
    }
}

#[derive(Default, Clone)]
pub struct FutureBoxSend<T> {
    inner: Rc<CallbackFutureInner<T>>,
}

impl<T> FutureBoxSend<T> {
    fn new(inner: Rc<CallbackFutureInner<T>>) -> FutureBoxSend<T> {
        FutureBoxSend { inner }
    }

    pub fn publish(&self, result: T) {
        self.inner.result.set(Some(result));
        if let Some(w) = self.inner.waker.take() {
            w.wake()
        }
    }
}

#[derive(Default, Clone)]
pub struct FutureBox<T> {
    inner: Rc<CallbackFutureInner<T>>,
}

impl<T> FutureBox<T> {
    pub fn new() -> (FutureBoxSend<T>, FutureBox<T>) {
        let inner = CallbackFutureInner::new();
        let sender = FutureBoxSend::new(inner.clone());
        let future = FutureBox { inner };
        (sender, future)
    }
}

impl<T> Future for FutureBox<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.result.take() {
            Some(x) => Poll::Ready(x),
            None => {
                self.inner.waker.set(Some(cx.waker().clone()));
                Poll::Pending
            }
        }
    }
}
