use std::cell::{RefCell};
use std::future::Future;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::driver_module::modules::interval::DriverBrowserInterval;

#[inline]
pub fn spawn_local<F>(interval: DriverBrowserInterval, future: F)
where
    F: Future<Output = ()> + 'static,
{
    Task::spawn(interval, Box::pin(future));
}

struct Inner {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
    waker: Waker,
}

pub(crate) struct Task {
    inner: RefCell<Option<Inner>>,
    interval: DriverBrowserInterval,
}

impl Task {
    pub(crate) fn spawn(interval: DriverBrowserInterval, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        let this = Rc::new(Self {
            inner: RefCell::new(None),
            interval,
        });

        let waker = unsafe { Waker::from_raw(Task::into_raw_waker(Rc::clone(&this))) };

        *this.inner.borrow_mut() = Some(Inner { future, waker });

        Task::wake_by_ref(&this);
    }

    fn wake_by_ref(this: &Rc<Self>) {
        let this_clone = this.clone();

        this.interval.set_timeout_and_detach(0, move |_| {
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
        let mut borrow = self.inner.borrow_mut();

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
    }
}
