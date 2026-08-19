use std::rc::Rc;

use crate::{
    Computed, DomComment, DomNode, Value, driver_module::get_driver_dom, struct_mut::ValueMut,
};

/// Render a computed value as a DOM node.
pub fn render_value<T: Clone + PartialEq + 'static>(
    computed: Computed<T>,
    render: impl Fn(T) -> DomNode + 'static,
) -> DomNode {
    render_value_option(computed, move |value| -> Option<DomNode> {
        Some(render(value))
    })
}

/// Render a computed value as an optional DOM node.
pub fn render_value_option<T: Clone + PartialEq + 'static>(
    computed: Computed<T>,
    render: impl Fn(T) -> Option<DomNode> + 'static,
) -> DomNode {
    let render = Rc::new(render);

    DomComment::new_marker("v", move |parent_id, comment_id, content| {
        let current_node: ValueMut<Option<DomNode>> = ValueMut::new(None);

        Some(computed.clone().subscribe({
            let render = render.clone();
            let content = content.clone();

            move |value| {
                let new_element = render(value).inspect(|new_element| {
                    get_driver_dom().insert_before(
                        parent_id,
                        new_element.id_dom(),
                        Some(comment_id),
                    );
                });

                content.set(new_element.iter().map(|node| node.id_dom()).collect());

                current_node.change(|current| {
                    *current = new_element;
                });
            }
        }))
    })
    .into()
}

impl<T: Clone + PartialEq + 'static> Computed<T> {
    /// Render value inside this [`Computed`]. See [`Value::render_value()`] for examples.
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        render_value(self.clone(), render)
    }

    /// Render optional value inside this [`Computed`]. See [`Value::render_value_option()`] for examples.
    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        render_value_option(self.clone(), render)
    }
}

impl<T: Clone + PartialEq + 'static> Value<T> {
    /// Render value (reactively transforms `T` into `DomNode`)
    ///
    /// See [`computed_tuple`](crate::computed_tuple) if you want to render multiple values.
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
    pub fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        self.to_computed().render_value(render)
    }

    /// Render optional value (reactively transforms `T` into `Option<DomNode>`)
    ///
    /// See [`computed_tuple`](crate::computed_tuple) if you want to render multiple values.
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
    pub fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        self.to_computed().render_value_option(render)
    }
}

/// Render a [`Value`] or [`Computed`] as a [`DomNode`].
///
/// Prefer the inherent methods [`Value::render_value`] / [`Computed::render_value`].
/// This trait is useful in generic code.
pub trait RenderValue<T> {
    fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode;
    fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode;
}

impl<T: Clone + PartialEq + 'static> RenderValue<T> for Computed<T> {
    fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        Computed::render_value(self, render)
    }

    fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        Computed::render_value_option(self, render)
    }
}

impl<T: Clone + PartialEq + 'static> RenderValue<T> for Value<T> {
    fn render_value(&self, render: impl Fn(T) -> DomNode + 'static) -> DomNode {
        Value::render_value(self, render)
    }

    fn render_value_option(&self, render: impl Fn(T) -> Option<DomNode> + 'static) -> DomNode {
        Value::render_value_option(self, render)
    }
}
