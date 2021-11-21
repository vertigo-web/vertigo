use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::{
    Driver, Instant, InstantType, Resource,
    computed::{Value},
    utils::BoxRefCell,
};

/// Function that updates cached value
pub trait Loader<T: PartialEq> = Fn(Driver) -> LoaderResult<T> + 'static;
pub type LoaderResult<T> = Pin<Box<dyn Future<Output=Resource<T>>>>;

pub struct CachedValue<T: PartialEq + 'static> {
    value: Value<Resource<T>>,
    updated_at: BoxRefCell<Instant>,
    set: BoxRefCell<bool>,
}

pub struct LazyCache<T: PartialEq + 'static> {
    res: Rc<CachedValue<T>>,
    max_age: InstantType,
    loader: Rc<dyn Loader<T>>,
    queued: Rc<BoxRefCell<bool>>,
    driver: Driver,
}

impl<T: PartialEq> CachedValue<T> {
    fn get_value(&self) -> Rc<Resource<T>> {
        self.value.get_value()
    }

    pub fn set_value(&self, value: Resource<T>) {
        self.value.set_value(value);
        self.updated_at.change((), |val, _| *val = val.refresh());
        if !self.is_set() {
            self.set.change((), |val, _| *val = true);
        }
    }

    fn age(&self) -> InstantType {
        self.updated_at.get(|val| val.seconds_elapsed())
    }

    fn is_set(&self) -> bool {
        self.set.get(|val| *val)
    }
}

impl<T: PartialEq> LazyCache<T> {
    pub fn new(driver: &Driver, max_age: InstantType, loader: impl Loader<T>) -> Self {
        Self {
            res: Rc::new(CachedValue {
                value: driver.new_value(Resource::Loading),
                updated_at: BoxRefCell::new(driver.now(), "CachedValue::updated_at"),
                set: BoxRefCell::new(false, "CachedValue::set"),
            }),
            max_age,
            loader: Rc::new(loader),
            queued: Rc::new(BoxRefCell::new(false, "LazyCache::queued")),
            driver: driver.clone(),
        }
    }

    pub fn result<F: Future<Output=Resource<T>> + 'static>(future: F) -> LoaderResult<T> {
        Box::pin(future)
    }

    pub fn get_value(&self) -> Rc<Resource<T>> {
        if self.needs_update() {
            self.force_update()
        }
        self.res.get_value()
    }

    pub fn force_update(&self) {
        let loader = self.loader.clone();
        let res = self.res.clone();
        let queued = self.queued.clone();
        queued.change((), |x, _| *x = true);
        let driver = self.driver.clone();

        self.driver.spawn(async move {
            res.set_value(Resource::Loading);
            res.set_value(loader(driver).await);
            queued.change((), |x, _| *x = false);
        })
    }

    pub fn needs_update(&self) -> bool {
        if self.is_loading_queued() {
            return false
        }

        if !self.res.is_set() {
            return true
        }

        self.res.age() >= self.max_age
    }

    fn is_loading_queued(&self) -> bool {
        self.queued.get(|val| *val)
    }
}

impl<T: PartialEq> PartialEq for CachedValue<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.value.eq(&rhs.value)
    }
}

impl<T: PartialEq> PartialEq for LazyCache<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.res.eq(&rhs.res)
    }
}
