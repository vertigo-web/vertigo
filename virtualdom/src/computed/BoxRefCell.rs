use std::cell::{RefCell/*, Ref*/};

struct AccessGuard {
    lock: bool,
}

impl AccessGuard {
    fn new() -> AccessGuard {
        AccessGuard {
            lock: false
        }
    }
}

struct AccessGuardPass {}

impl AccessGuardPass {
    fn new() -> AccessGuardPass {
        AccessGuardPass {}
    }

    fn off(self) {}
}

impl Drop for AccessGuardPass {
    fn drop(&mut self) {
        guardStatic.with(|inst| {
            let mut inst = inst.borrow_mut();

            if inst.lock == false {
                panic!("Aktualnie zdjęta jest blokada");
            }

            inst.lock = false;
        });
    }
}

thread_local! {
    static guardStatic: RefCell<AccessGuard> = RefCell::new(AccessGuard::new());
}

fn lockAccess() -> AccessGuardPass {
    guardStatic.with(|inst| {
        let mut inst = inst.borrow_mut();

        if inst.lock {
            panic!("Aktualnie jest zalozona blokada");
        }

        inst.lock = true;
    });

    AccessGuardPass::new()
}

pub struct BoxRefCell<T> {
    value: RefCell<T>,
}

impl<T> BoxRefCell<T> {
    pub fn new(value: T) -> BoxRefCell<T> {
        BoxRefCell {
            value: RefCell::new(value),
        }
    }

    pub fn get<R>(&self, getter: fn(&T) -> R) -> R {
        let guard = lockAccess();
        let value = self.value.borrow();
        let state = &*value;
        let result = getter(&state);
        guard.off();
        result
    }

    // pub fn getRef(&self) -> Ref<T> {
    //     self.value.borrow()
    // }

    pub fn getWithContext<D, R>(&self, data: D, getter: fn(&T, D) -> R) -> R {
        let guard = lockAccess();
        let value = self.value.borrow();
        let state = &*value;
        let result = getter(&state, data);
        guard.off();
        result
    }

    pub fn changeNoParams<R>(&self, changeFn: fn(&mut T) -> R) -> R {
        let guard = lockAccess();
        let value = self.value.borrow_mut();
        let mut state = value;
        let result = changeFn(&mut state);
        guard.off();
        result
    }

    pub fn change<D, R>(&self, data: D, changeFn: fn(&mut T, D) -> R) -> R {
        let guard = lockAccess();
        let value = self.value.borrow_mut();
        let mut state = value;
        let result = changeFn(&mut state, data);
        guard.off();
        result
    }
}