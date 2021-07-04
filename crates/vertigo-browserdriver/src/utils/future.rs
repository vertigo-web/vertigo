use std::task::{Waker, Context, Poll};
use std::cell::Cell;
use std::future::Future;
use std::rc::Rc;
use std::pin::Pin;

//https://users.rust-lang.org/t/how-to-receive-a-async-callback/40110/3
//https://users.rust-lang.org/t/can-you-turn-a-callback-into-a-future-into-async-await/49378/16
//https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=10cd4b012f678705c9d78fed5ca81857

#[derive(Default)]
pub(crate) struct CallbackFutureInner<T> {
    waker: Cell<Option<Waker>>,
    result: Cell<Option<T>>
}

impl<T> CallbackFutureInner<T> {
    pub fn new() -> Rc<CallbackFutureInner<T>> {
        Rc::new(CallbackFutureInner {
            waker: Cell::new(None),
            result: Cell::new(None),
        })
    }
}

#[derive(Default,Clone)]
pub(crate) struct CbFutureSend<T> {
    inner: Rc<CallbackFutureInner<T>>
}

impl<T> CbFutureSend<T> {
    pub(crate) fn new(inner: Rc<CallbackFutureInner<T>>) -> CbFutureSend<T> {
        CbFutureSend {
            inner
        }
    }

    pub fn publish(&self, result:T) {
        self.inner.result.set(Some(result));
        self.inner.waker.take().map(|w| w.wake());
    }
}


#[derive(Default,Clone)]
pub(crate) struct CbFutureReceiver<T> {
    inner: Rc<CallbackFutureInner<T>>
}

impl<T> CbFutureReceiver<T> {
    pub fn new(inner: Rc<CallbackFutureInner<T>>) -> CbFutureReceiver<T> {
        CbFutureReceiver {
            inner
        }
    }
}

impl<T> Future for CbFutureReceiver<T> {
    type Output=T;
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

pub(crate) fn new_future<T>() -> (CbFutureSend<T>, CbFutureReceiver<T>) {
    let inner = CallbackFutureInner::new();
    let sender = CbFutureSend::new(inner.clone());
    let future = CbFutureReceiver::new(inner);
    (sender, future)
}
