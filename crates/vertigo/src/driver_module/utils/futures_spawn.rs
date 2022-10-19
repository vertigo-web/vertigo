use std::future::Future;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::ApiImport;
use crate::struct_mut::ValueMut;

#[inline]
pub fn spawn_local<F>(api: ApiImport, future: F)
where
    F: Future<Output = ()> + 'static,
{
    Task::spawn(api, Box::pin(future));
}

struct Inner {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

pub(crate) struct Task {
    inner: ValueMut<Option<Inner>>,
    api: ApiImport
}

impl Task {
    pub(crate) fn spawn(api: ApiImport, future: Pin<Box<dyn Future<Output = ()>>>) {
        let this = Rc::new(Self {
            inner: ValueMut::new(None),
            api,
        });

        let waker = unsafe { Waker::from_raw(Task::into_raw_waker(Rc::clone(&this))) };

        this.inner.set(Some(Inner { future, waker }));

        Task::wake_by_ref(&this);
    }

    fn wake_by_ref(this: &Rc<Self>) {
        let this_clone = this.clone();

        this.api.set_timeout_and_detach(0, move || {
            this_clone.run();
        });
    }

    unsafe fn into_raw_waker(this: Rc<Self>) -> RawWaker {
        unsafe fn raw_clone(ptr: *const ()) -> RawWaker {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const Task));
            Task::into_raw_waker((*ptr).clone())
        }

        unsafe fn raw_wake(ptr: *const ()) {
            let ptr = Rc::from_raw(ptr as *const Task);
            Task::wake_by_ref(&ptr);
        }

        unsafe fn raw_wake_by_ref(ptr: *const ()) {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const Task));
            Task::wake_by_ref(&ptr);
        }

        unsafe fn raw_drop(ptr: *const ()) {
            drop(Rc::from_raw(ptr as *const Task));
        }

        const VTABLE: RawWakerVTable =
            RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

        RawWaker::new(Rc::into_raw(this) as *const (), &VTABLE)
    }

    fn run(&self) {
        self.inner.change(|borrow| {
            let inner = match borrow.as_mut() {
                Some(inner) => inner,
                None => return,
            };

            let poll = {
                let mut cx = Context::from_waker(&inner.waker);
                inner.future.as_mut().poll(&mut cx)
            };

            if poll.is_ready() {
                *borrow = None;
            }
        });
    }
}
