use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::{
    Driver, Instant, InstantType, Resource,
    computed::Value, struct_mut::ValueMut,
};

/// Function that updates cached value
pub trait Loader<T: PartialEq> = Fn(Driver) -> LoaderResult<T> + 'static;

/// Type that should be returned from [Loader] which is in fact a future [resource](Resource).
pub type LoaderResult<T> = Pin<Box<dyn Future<Output = Resource<T>>>>;

/// Value that [LazyCache] holds.
pub struct CachedValue<T: PartialEq + 'static> {
    value: Value<Resource<T>>,
    updated_at: ValueMut<Instant>,
    set: ValueMut<bool>,
}

/// A structure similar to Value but supports Loading/Error states and automatic refresh
/// after defined amount of time.
///
/// ```rust,ignore
/// use vertigo::{Computed, Driver, LazyCache, SerdeRequest};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(PartialEq, Serialize, Deserialize, SerdeRequest)]
/// pub struct Model {
///     id: i32,
///     name: String,
/// }
///
/// #[derive(PartialEq)]
/// pub struct TodoState {
///     driver: Driver,
///     posts: LazyCache<Vec<Model>>,
/// }
///
/// impl TodoState {
///     pub fn new(driver: Driver) -> Computed<TodoState> {
///         let posts = LazyCache::new(&driver, 300, move |driver: Driver| {
///             let request =  driver
///                 .request("https://some.api/posts")
///                 .get();
///
///             LazyCache::result(async move {
///                 request.await.into(|status, body| {
///                     if status == 200 {
///                         Some(body.into_vec::<Model>())
///                     } else {
///                         None
///                     }
///                 })
///             })
///         });
///
///         driver.new_computed_from(TodoState {
///             driver: driver.clone(),
///             posts,
///         })
///     }
/// }
/// ```
///
/// See ["todo" example](../src/vertigo_demo/app/todo/mod.rs.html) in vertigo-demo package for more.
pub struct LazyCache<T: PartialEq + 'static> {
    res: Rc<CachedValue<T>>,
    max_age: InstantType,
    loader: Rc<dyn Loader<T>>,
    queued: Rc<ValueMut<bool>>,
    driver: Driver,
}

impl<T: PartialEq> Clone for LazyCache<T> {
    fn clone(&self) -> Self {
        LazyCache {
            res: self.res.clone(),
            max_age: self.max_age,
            loader: self.loader.clone(),
            queued: self.queued.clone(),
            driver: self.driver.clone()
        }
    }
}

impl<T: PartialEq> CachedValue<T> {
    fn get_value(&self) -> Rc<Resource<T>> {
        self.value.get_value()
    }

    pub fn set_value(&self, value: Resource<T>) {
        self.value.set_value(value);
        let current_updated_at = self.updated_at.get();
        self.updated_at.set(current_updated_at.refresh());
        // self.updated_at.change((), |val, _| *val = val.refresh());
        if !self.is_set() {
            self.set.set(true);
        }
    }

    fn age(&self) -> InstantType {
        self.updated_at.get().seconds_elapsed()
    }

    fn is_set(&self) -> bool {
        self.set.get()
    }
}

impl<T: PartialEq> LazyCache<T> {
    pub fn new(driver: &Driver, max_age: InstantType, loader: impl Loader<T>) -> Self {
        Self {
            res: Rc::new(CachedValue {
                value: driver.new_value(Resource::Loading),
                updated_at: ValueMut::new(driver.now()),
                set: ValueMut::new(false),
            }),
            max_age,
            loader: Rc::new(loader),
            queued: Rc::new(ValueMut::new(false)),
            driver: driver.clone(),
        }
    }

    pub fn result<F: Future<Output = Resource<T>> + 'static>(future: F) -> LoaderResult<T> {
        Box::pin(future)
    }

    pub fn get_value(&self) -> Rc<Resource<T>> {
        if self.needs_update() {
            self.force_update(true)
        }
        self.res.get_value()
    }

    pub fn force_update(&self, with_loading: bool) {
        let loader = self.loader.clone();
        let res = self.res.clone();
        let queued = self.queued.clone();
        queued.set(true);
        let driver = self.driver.clone();

        self.driver.spawn(async move {
            if with_loading {
                res.set_value(Resource::Loading);
            }
            res.set_value(loader(driver).await);
            queued.set(false);
        })
    }

    pub fn needs_update(&self) -> bool {
        if self.is_loading_queued() {
            return false;
        }

        if !self.res.is_set() {
            return true;
        }

        self.res.age() >= self.max_age
    }

    fn is_loading_queued(&self) -> bool {
        self.queued.get()
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
