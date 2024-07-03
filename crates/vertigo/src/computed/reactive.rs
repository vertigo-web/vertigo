use crate::{Context, Value};

/// A trait that tells `Something<T>` is behaving like a [`Value<T>`].
///
/// Generic components should accept [`Reactive<T>`] instead of [`Value<T>`] to be more flexible,
/// so that types wrapping a Value can implement Reactive and be used in those components.
///
/// Automatically implemented for the [Value] itself.
///
/// Example:
///
/// ```rust
/// use vertigo::{component, EmbedDom, Computed, Context, dom, DomNode, Reactive, Value, transaction};
///
/// // Define reactive object with `Value` wrapped
/// #[derive(Clone, PartialEq)]
/// struct Lisper {
///     inner: Value<String>
/// }
///
/// // Implement `EmbedDom` so it can be easily rendered
/// impl EmbedDom for Lisper {
///     fn embed(self) -> DomNode {
///         self.inner.embed()
///     }
/// }
///
/// // Implement `Reactive`
/// impl Reactive<String> for Lisper {
///     fn set(&self, val: String) {
///         let lisp = val.replace('r', "w");
///         self.inner.set(lisp)
///     }
///
///     fn get(&self, ctx: &Context) -> std::string::String {
///         self.inner.get(ctx)
///     }
///
///     fn change(&self, change_fn: impl FnOnce(&mut String)) {
///         self.inner.change(|val| {
///             change_fn(val);
///             *val = val.replace('r', "w");
///         });
///     }
/// }
///
/// // Exemplary generic component
/// #[component]
/// fn Print<R>(saying: R)
/// where
///    R: Reactive<String> + EmbedDom
/// {
///     dom! {
///         <quote>{saying}</quote>
///     }
/// }
///
/// // Create reactive object
/// let lisper = Lisper {
///     inner: Value::new("".to_string())
/// };
///
/// // Specialize Print component
/// type PrintL = Print::<Lisper>;
///
/// // Use reactive object in generic component
/// let _ = dom! {
///     <div>
///         <PrintL saying={lisper.clone()} />
///     </div>
/// };
///
/// lisper.set("Eating raisins and radishes".to_string());
///
/// transaction(|context| {
///     assert_eq!(lisper.get(context), "Eating waisins and wadishes");
/// })
/// ```
pub trait Reactive<T>: PartialEq {
    fn set(&self, value: T);
    fn get(&self, context: &Context) -> T;
    fn change(&self, change_fn: impl FnOnce(&mut T));
}

impl<T> Reactive<T> for Value<T>
where
    T: Clone + PartialEq + 'static
{
    fn set(&self, value: T) {
        Value::set(self, value)
    }

    fn get(&self, context: &Context) -> T {
        Value::get(self, context)
    }

    fn change(&self, change_fn: impl FnOnce(&mut T)) {
        Value::change(self, change_fn)
    }
}
