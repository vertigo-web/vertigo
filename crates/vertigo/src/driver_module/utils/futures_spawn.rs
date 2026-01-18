use std::{
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    rc::Rc,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use crate::{computed::struct_mut::ValueMut, driver_module::api::api_timers};

#[inline]
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    Task::spawn(Box::pin(future));
}

struct Inner {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

pub(crate) struct Task {
    inner: ValueMut<Option<Inner>>,
}

impl Task {
    pub(crate) fn spawn(future: Pin<Box<dyn Future<Output = ()>>>) {
        let this = Rc::new(Self {
            inner: ValueMut::new(None),
        });

        let waker = unsafe { Waker::from_raw(Task::into_raw_waker(Rc::clone(&this))) };

        this.inner.set(Some(Inner { future, waker }));

        Task::wake_by_ref(&this);
    }

    fn wake_by_ref(this: &Rc<Self>) {
        let this_clone = this.clone();

        api_timers().set_timeout_and_detach(0, move || {
            this_clone.run();
        });
    }

    fn into_raw_waker(this: Rc<Self>) -> RawWaker {
        fn raw_clone(ptr: *const ()) -> RawWaker {
            let ptr = ManuallyDrop::new(unsafe { Rc::from_raw(ptr as *const Task) });
            Task::into_raw_waker((*ptr).clone())
        }

        fn raw_wake(ptr: *const ()) {
            let ptr = unsafe { Rc::from_raw(ptr as *const Task) };
            Task::wake_by_ref(&ptr);
        }

        fn raw_wake_by_ref(ptr: *const ()) {
            let ptr = ManuallyDrop::new(unsafe { Rc::from_raw(ptr as *const Task) });
            Task::wake_by_ref(&ptr);
        }

        fn raw_drop(ptr: *const ()) {
            drop(unsafe { Rc::from_raw(ptr as *const Task) });
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
