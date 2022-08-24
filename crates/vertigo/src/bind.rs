
use std::future::Future;
use crate::{get_driver, Context};

pub fn bind<T1: Clone>(param1: &T1) -> Bind1<T1> {
    let param1 = param1.clone();

    Bind1 {
        param1
    }
}

pub struct Bind1<T1> {
    param1: T1,
}

impl<T1: Clone> Bind1<T1> {
    pub fn and<T2: Clone>(self, param2: &T2) -> Bind2<T1, T2> {
        Bind2 {
            param1: self.param1,
            param2: param2.clone(),
        }
    }

    pub fn call<R>(self, fun: fn(&Context, &T1) -> R) -> impl Fn() -> R {
        let Self { param1 } = self;

        move || -> R {
            let context = Context::new();
            fun(&context, &param1)
        }
    }

    pub fn call_param<T2, R>(self, fun: fn(&Context, &T1, T2) -> R) -> impl Fn(T2) -> R {
        let Self { param1 } = self;

        move |param2: T2| -> R {
            let context = Context::new();
            fun(&context, &param1, param2)
        }
    }

    pub fn spawn<
        Fut: Future<Output=Context> + 'static
    >(self, fun: fn(Context, T1) -> Fut) -> impl Fn() {
        let Self { param1 } = self;

        move || {
            let context = Context::new();
            let param1 = param1.clone();
            let future = fun(context, param1);
            get_driver().spawn(async move {
                future.await;
            });
        }
    }
}

pub fn bind2<T1: Clone, T2: Clone>(param1: &T1, param2: &T2) -> Bind2<T1, T2> {
    Bind2 {
        param1: param1.clone(),
        param2: param2.clone()
    }
}

pub struct Bind2<T1, T2> {
    param1: T1,
    param2: T2,
}

impl<T1: Clone, T2: Clone> Bind2<T1, T2> {
    pub fn and<T3: Clone>(self, param3: &T3) -> Bind3<T1, T2, T3> {
        Bind3 {
            param1: self.param1,
            param2: self.param2,
            param3: param3.clone(),
        }
    }

    pub fn call<R>(self, fun: fn(&Context, &T1, &T2) -> R) -> impl Fn() -> R {
        let Self { param1, param2 } = self;

        move || -> R {
            let context = Context::new();
            fun(&context, &param1, &param2)
        }
    }

    pub fn call_param<T3, R>(self, fun: fn(&Context, &T1, &T2, T3) -> R) -> impl Fn(T3) -> R {
        let Self { param1, param2 } = self;

        move |param3: T3| -> R {
            let context = Context::new();
            fun(&context, &param1, &param2, param3)
        }
    }

    pub fn spawn<
        Fut: Future<Output=Context> + 'static
    >(self, fun: fn(Context, T1, T2) -> Fut) -> impl Fn() {
        let Self { param1, param2 } = self;

        move || {
            let context = Context::new();
            let param1 = param1.clone();
            let param2 = param2.clone();
            let future = fun(context, param1, param2);
            get_driver().spawn(async move {
                future.await;
            });
        }
    }
}

pub fn bind3<T1: Clone, T2: Clone, T3: Clone>(param1: &T1, param2: &T2, param3: &T3) -> Bind3<T1, T2, T3> {
    Bind3 {
        param1: param1.clone(),
        param2: param2.clone(),
        param3: param3.clone(),
    }
}

pub struct Bind3<T1, T2, T3> {
    param1: T1,
    param2: T2,
    param3: T3,
}

impl<T1: Clone, T2: Clone, T3: Clone> Bind3<T1, T2, T3> {
    pub fn and<T4: Clone>(self, param4: &T4) -> Bind4<T1, T2, T3, T4> {
        Bind4 {
            param1: self.param1,
            param2: self.param2,
            param3: self.param3,
            param4: param4.clone(),
        }
    }

    pub fn call<R>(self, fun: fn(&Context, &T1, &T2, &T3) -> R) -> impl Fn() -> R {
        let Self { param1, param2, param3 } = self;

        move || -> R {
            let context = Context::new();
            fun(&context, &param1, &param2, &param3)
        }
    }

    pub fn call_param<T4, R>(self, fun: fn(&Context, &T1, &T2, &T3, T4) -> R) -> impl Fn(T4) -> R {
        let Self { param1, param2, param3 } = self;

        move |param4: T4| -> R {
            let context = Context::new();
            fun(&context, &param1, &param2, &param3, param4)
        }
    }

    pub fn spawn<
        Fut: Future<Output=Context> + 'static
    >(self, fun: fn(Context, T1, T2, T3) -> Fut) -> impl Fn() {
        let Self { param1, param2, param3 } = self;

        move || {
            let context = Context::new();
            let param1 = param1.clone();
            let param2 = param2.clone();
            let param3 = param3.clone();
            let future = fun(context, param1, param2, param3);
            get_driver().spawn(async move {
                future.await;
            });
        }
    }
}

pub fn bind4<T1: Clone, T2: Clone, T3: Clone, T4: Clone>(param1: &T1, param2: &T2, param3: &T3, param4: &T4) -> Bind4<T1, T2, T3, T4> {
    Bind4 {
        param1: param1.clone(),
        param2: param2.clone(),
        param3: param3.clone(),
        param4: param4.clone(),
    }
}

pub struct Bind4<T1, T2, T3, T4> {
    param1: T1,
    param2: T2,
    param3: T3,
    param4: T4,
}

impl<T1: Clone, T2: Clone, T3: Clone, T4: Clone> Bind4<T1, T2, T3, T4> {
    pub fn call<R>(self, fun: fn(&Context, &T1, &T2, &T3, &T4) -> R) -> impl Fn() -> R {
        let Self { param1, param2, param3, param4 } = self;

        move || -> R {
            let context = Context::new();
            fun(&context, &param1, &param2, &param3, &param4)
        }
    }

    pub fn call_param<T5, R>(self, fun: fn(&Context, &T1, &T2, &T3, &T4, T5) -> R) -> impl Fn(T5) -> R {
        let Self { param1, param2, param3, param4 } = self;

        move |param5: T5| -> R {
            let context = Context::new();
            fun(&context, &param1, &param2, &param3, &param4, param5)
        }
    }

    pub fn spawn<
        Fut: Future<Output=Context> + 'static,
    >(self, fun: fn(Context, T1, T2, T3, T4) -> Fut) -> impl Fn() {
        let Self { param1, param2, param3, param4 } = self;

        move || {
            let context = Context::new();
            let param1 = param1.clone();
            let param2 = param2.clone();
            let param3 = param3.clone();
            let param4 = param4.clone();
            let future = fun(context, param1, param2, param3, param4);
            get_driver().spawn(async move {
                future.await;
            });
        }
    }
}

