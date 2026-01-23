use std::rc::Rc;

use vertigo_macro::bind;

use crate::{Context, DomNode, ToComputed, computed::value_inner::ValueInner};

use super::{Computed, DropResource, GraphId, dependencies::get_dependencies};

/// A reactive value. Basic building block of app state.
///
/// Can be read or written.
///
/// ```rust
/// use vertigo::{Value, transaction};
///
/// let value = Value::new(5);
///
/// transaction(|context| {
///     assert_eq!(value.get(context), 5);
/// });
///
/// value.set(10);
///
/// transaction(|context| {
///     assert_eq!(value.get(context), 10);
/// });
/// ```
///
#[derive(Clone)]
pub struct Value<T: Clone + PartialEq + 'static> {
    inner: Rc<ValueInner<T>>,
}

impl<T: Clone + PartialEq + Default + 'static> Default for Value<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone + PartialEq> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id.eq(&other.inner.id)
    }
}

impl<T: Clone + PartialEq + 'static> ToComputed<T> for Value<T> {
    fn to_computed(&self) -> Computed<T> {
        self.to_computed()
    }
}

impl<T: Clone + PartialEq + 'static> ToComputed<T> for &Value<T> {
    fn to_computed(&self) -> Computed<T> {
        (*self).to_computed()
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
    pub fn new(value: T) -> Self {
        Value {
            inner: Rc::new(ValueInner::new(value)),
        }
    }

    /// Create a value that is connected to a generator, where `value` parameter is a starting value,
    /// and `create` function takes care of updating it.
    ///
    /// See [Router implementation](../src/vertigo/router.rs.html#97) for example example.
    pub fn with_connect<F>(value: T, create: F) -> Computed<T>
    where
        F: Fn(&Value<T>) -> DropResource + 'static,
    {
        let value = Value::new(value);
        let value_clone = value.clone();
        value
            .to_computed()
            .when_connect(move || create(&value_clone))
    }

    /// Get the value.
    ///
    /// Use this in callbacks. You can get [Context] object using [transaction](crate::transaction) function.
    /// During rendering you should directly embed the `Value` in [dom!](crate::dom) or use [.render_value()](Value::render_value) method.
    ///
    /// Returned `T` is cloned - it's not reactive.
    pub fn get(&self, context: &Context) -> T {
        context.add_parent(self.inner.id, self.inner.clone());
        self.inner.get()
    }

    /// Reactively convert `Value` into [Computed] with provided transformation function applied.
    pub fn map<K: Clone + 'static, F: 'static + Fn(T) -> K>(&self, fun: F) -> Computed<K> {
        Computed::from({
            let myself = self.clone();
            move |context| fun(myself.get(context))
        })
    }

    pub fn id(&self) -> GraphId {
        self.inner.id
    }

    /// Reactively convert the `Value`` into [Computed] without any mapping.
    pub fn to_computed(&self) -> Computed<T> {
        let myself = self.clone();

        Computed::from(move |context| myself.get(context))
    }

    pub fn change(&self, change_fn: impl FnOnce(&mut T)) {
        get_dependencies().transaction(|ctx| {
            let mut value = self.get(ctx);
            change_fn(&mut value);
            self.set(value);
        });
    }

    pub fn set(&self, value: T) {
        get_dependencies().transaction(|_| {
            let need_refresh = self.inner.set(value);
            if need_refresh {
                get_dependencies().report_set(self.inner.id);
            }
        });
    }

    /// Render value (reactively transforms `T` into `DomNode`)
    ///
    /// See [computed_tuple](macro.computed_tuple.html) if you want to render multiple values in a handy way.
    ///
    /// ```rust
    /// use vertigo::{dom, Value};
    ///
    /// let my_value = Value::new(5);
    ///
    /// let element = my_value.render_value(|bare_value| dom! { <div>{bare_value}</div> });
    ///
    /// dom! {
    ///     <div>
    ///         {element}
    ///     </div>
    /// };
    /// ```
    ///
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        self.to_computed().render_value(render)
    }

    /// Render optional value (reactively transforms `Option<T>` into `Option<DomNode>`)
    ///
    /// See [computed_tuple](macro.computed_tuple.html) if you want to render multiple values in a handy way.
    ///
    /// ```rust
    /// use vertigo::{dom, Value};
    ///
    /// let value1 = Value::new(Some(5));
    /// let value2 = Value::new(None::<i32>);
    ///
    /// let element1 = value1.render_value_option(|bare_value|
    ///     bare_value.map(|value| dom! { <div>{value}</div> })
    /// );
    /// let element2 = value2.render_value_option(|bare_value|
    ///     match bare_value {
    ///         Some(value) => Some(dom! { <div>{value}</div> }),
    ///         None => Some(dom! { <div>"default"</div> }),
    ///     }
    /// );
    ///
    /// dom! {
    ///     <div>
    ///         {element1}
    ///         {element2}
    ///     </div>
    /// };
    /// ```
    ///
    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        self.to_computed().render_value_option(render)
    }

    pub fn add_event(&self, callback: impl Fn(T) + 'static) -> DropResource {
        self.inner.add_event(callback)
    }

    pub fn synchronize<R: ValueSynchronize<T> + Clone + 'static>(&self) -> (R, DropResource) {
        let init_value = self.inner.get();
        let synchronize_target = R::new(init_value);

        let drop_synchronize = self.add_event(bind!(synchronize_target, |current| {
            synchronize_target.set(current);
        }));

        (synchronize_target, drop_synchronize)
    }
}

pub trait ValueSynchronize<T: PartialEq + Clone + 'static>: Sized {
    fn new(value: T) -> Self;
    fn set(&self, value: T);
}
