use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;

use crate::computed::context::Context;
use crate::{get_driver, Computed};
use crate::{
    Instant, InstantType, Resource,
    computed::Value, struct_mut::ValueMut,
};

use crate::fetch::pinboxfut::PinBoxFuture;

fn get_unique_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Value that [LazyCache] holds.
struct CachedValue<T> {
    id: u64,
    value: Value<Resource<Rc<T>>>,
    updated_at: ValueMut<Instant>,
    set: ValueMut<bool>,
}

impl<T: 'static> CachedValue<T> {
    fn get_value(&self, context: &Context) -> Resource<Rc<T>> {
        self.value.get(context)
    }

    pub fn set_value(&self, value: Resource<T>) {
        self.value.set(value.map(|value| Rc::new(value)));
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

/// A structure similar to Value but supports Loading/Error states and automatic refresh
/// after defined amount of time.
///
/// ```rust
/// use vertigo::{get_driver, Computed, LazyCache, SerdeRequest, Resource};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, SerdeRequest, PartialEq, Clone)]
/// pub struct Model {
///     id: i32,
///     name: String,
/// }
///
/// pub struct TodoState {
///     posts: LazyCache<Vec<Model>>,
/// }
///
/// impl TodoState {
///     pub fn new() -> Self {
///         let posts = LazyCache::new(300, move || {
///             let request = get_driver()
///                 .request("https://some.api/posts")
///                 .get();
///
///             async move {
///                 request.await.into(|status, body| {
///                     if status == 200 {
///                         Some(body.into_vec::<Model>())
///                     } else {
///                         None
///                     }
///                 })
///             }
///         });
///
///         TodoState {
///             posts
///         }
///     }
/// }
/// ```
///
/// See ["todo" example](../src/vertigo_demo/app/todo/mod.rs.html) in vertigo-demo package for more.
pub struct LazyCache<T: 'static> {
    res: Rc<CachedValue<T>>,
    max_age: InstantType,
    loader: Rc<dyn Fn() -> PinBoxFuture<Resource<T>>>,
    queued: Rc<ValueMut<bool>>,
}

impl<T: 'static> Debug for LazyCache<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct ("LazyCache")
            .field("max_age", &self.max_age)
            .field("queued", &self.queued)
            .finish()
    }
}

impl<T> Clone for LazyCache<T> {
    fn clone(&self) -> Self {
        LazyCache {
            res: self.res.clone(),
            max_age: self.max_age,
            loader: self.loader.clone(),
            queued: self.queued.clone(),
        }
    }
}

impl<T> PartialEq for LazyCache<T> {
    fn eq(&self, other: &Self) -> bool {
        self.res.id == other.res.id
    }
}

impl<T> LazyCache<T> {
    pub fn new<Fut: Future<Output = Resource<T>> + 'static, F: Fn() -> Fut + 'static>(max_age: InstantType, loader: F) -> Self {
        let loader_rc: Rc<dyn Fn() -> PinBoxFuture<Resource<T>>> = Rc::new(move || -> PinBoxFuture<Resource<T>> {
            Box::pin(loader())
        });

        Self {
            res: Rc::new(CachedValue {
                id: get_unique_id(),
                value: Value::new(Resource::Loading),
                updated_at: ValueMut::new(get_driver().now()),
                set: ValueMut::new(false),
            }),
            max_age,
            loader: loader_rc,
            queued: Rc::new(ValueMut::new(false)),
        }
    }

    pub fn get(&self, context: &Context) -> Resource<Rc<T>> {
        if self.needs_update() {
            self.force_update(true)
        }
        self.res.get_value(context)
    }

    pub fn force_update(&self, with_loading: bool) {
        let loader = self.loader.clone();
        let res = self.res.clone();
        let queued = self.queued.clone();
        queued.set(true);

        get_driver().spawn(async move {
            if with_loading {
                res.set_value(Resource::Loading);
            }
            res.set_value(loader().await);
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

    pub fn to_computed(&self) -> Computed<Resource<Rc<T>>> {
        Computed::from({
            let state = self.clone();
            move |context| {
                state.get(context)
            }
        })
    }
}
