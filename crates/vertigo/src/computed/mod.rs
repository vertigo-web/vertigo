mod auto_map;
mod computed_box;
pub mod context;
mod dependencies;
pub use dependencies::{get_dependencies, Dependencies};
mod drop_resource;
mod graph_id;
mod graph_value;
mod reactive;
pub mod struct_mut;
mod to_computed;
mod value;

#[cfg(test)]
mod tests;

pub use auto_map::AutoMap;
pub use computed_box::Computed;
pub use drop_resource::DropResource;
pub use graph_id::GraphId;
pub use graph_value::GraphValue;
pub use reactive::Reactive;
pub use to_computed::ToComputed;
pub use value::Value;

/// Allows to create `Computed<T1, T2, ...>` out of `Value<T1>`, `Value<T2>`, ...
///
/// # Examples
///
/// ```
/// use vertigo::{Value, computed_tuple};
///
/// let value1 = Value::new(true);
/// let value2 = Value::new(5);
/// let value3 = Value::new("Hello tuple!".to_string());
///
/// let my_tuple = computed_tuple!(value1, value2, value3);
///
/// vertigo::transaction(|ctx| {
///    assert!(my_tuple.get(ctx).0);
///    assert_eq!(my_tuple.get(ctx).1, 5);
///    assert_eq!(&my_tuple.get(ctx).2, "Hello tuple!");
/// });
/// ```
///
/// ```
/// use vertigo::{Value, computed_tuple};
///
/// let values = (Value::new(true), Value::new(5));
/// let value3 = Value::new("Hello tuple!".to_string());
///
/// let my_tuple = computed_tuple!(a => values.0, b => values.1, c => value3);
///
/// vertigo::transaction(|ctx| {
///    assert!(my_tuple.get(ctx).0);
///    assert_eq!(my_tuple.get(ctx).1, 5);
///    assert_eq!(&my_tuple.get(ctx).2, "Hello tuple!");
/// });
/// ```
#[macro_export]
macro_rules! computed_tuple {
    ($($arg: tt),*) => {{
        let ($($arg),*) = ($($arg.clone()),*);
        $crate::Computed::from(move |ctx| {
            ($($arg.get(ctx)),*)
        })
    }};

    ($($name: ident => $arg: expr),*) => {{
        let ($($name),*) = ($(($arg).clone()),*);
        $crate::Computed::from(move |ctx| {
            ($($name.get(ctx)),*)
        })
    }};
}
