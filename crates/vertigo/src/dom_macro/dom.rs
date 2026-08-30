use std::{collections::BTreeMap, rc::Rc};

use crate::{
    Computed, Css, DomComment, DomDisplay, DomElement, DomText, DropFileEvent, KeyDownEvent, Value,
    dom::{
        attr_value::AttrValue,
        callback::{Callback, Callback1},
        dom_node::DomNode,
        events::{ClickEvent, IntersectionEvent},
    },
};

/// Type interpreted as component's dynamic attributes groups
///
/// Be careful when using dynamic attributes, key-value type compatibility is checked
/// in runtime (errors logged into JS console) or ignored for `AttrValue` variant.
///
/// ```rust
/// use vertigo::{bind, component, dom, AttrGroup, Value};
///
/// #[component]
/// pub fn Input(value: Value<String>, input: AttrGroup) {
///     let on_input = bind!(value, |new_value: String| {
///         value.set(new_value);
///     });
///
///     dom! {
///         <input {value} {on_input} {..input} />
///     }
/// }
///
/// let value = Value::new("world".to_string());
///
/// dom! {
///     <div>
///        <Input {value} input:name="hello_value" input:id="my_input_1" />
///     </div>
/// };
/// ```
pub type AttrGroup = BTreeMap<String, AttrGroupValue>;

#[derive(Clone)]
pub enum AttrGroupValue {
    AttrValue(AttrValue),
    Css {
        css: Css,
        class_name: Option<String>,
    },
    HookKeyDown(Rc<Callback1<KeyDownEvent, bool>>),
    OnBlur(Rc<Callback<()>>),
    OnChange(Rc<Callback1<String, ()>>),
    OnClick(Rc<Callback1<ClickEvent, ()>>),
    OnDropfile(Rc<Callback1<DropFileEvent, ()>>),
    OnChangeFile(Rc<Callback1<DropFileEvent, ()>>),
    OnInput(Rc<Callback1<String, ()>>),
    OnIntersect(Rc<Callback1<IntersectionEvent, ()>>),
    OnKeyDown(Rc<Callback1<KeyDownEvent, bool>>),
    OnLoad(Rc<Callback<()>>),
    OnMouseDown(Rc<Callback<bool>>),
    OnMouseEnter(Rc<Callback<()>>),
    OnMouseLeave(Rc<Callback<()>>),
    OnMouseUp(Rc<Callback<bool>>),
    OnSubmit(Rc<Callback<()>>),
}

impl From<&Self> for AttrGroupValue {
    fn from(value: &Self) -> Self {
        value.to_owned()
    }
}

macro_rules! group_value_constructor {
    ($function:ident, $cb_type:ty, $variant:ident) => {
        pub fn $function(callback: impl Into<$cb_type>) -> Self {
            Self::$variant(Rc::new(callback.into()))
        }
    };
}

impl AttrGroupValue {
    pub fn css(css: impl Into<Css>, class_name: Option<String>) -> Self {
        Self::Css {
            css: css.into(),
            class_name,
        }
    }

    group_value_constructor!(hook_key_down, Callback1<KeyDownEvent, bool>, HookKeyDown);
    group_value_constructor!(on_blur, Callback<()>, OnBlur);
    group_value_constructor!(on_change, Callback1<String, ()>, OnChange);
    group_value_constructor!(on_click, Callback1<ClickEvent, ()>, OnClick);
    group_value_constructor!(on_dropfile, Callback1<DropFileEvent, ()>, OnDropfile);
    group_value_constructor!(on_change_file, Callback1<DropFileEvent, ()>, OnChangeFile);
    group_value_constructor!(on_input, Callback1<String, ()>, OnInput);
    group_value_constructor!(on_intersect, Callback1<IntersectionEvent, ()>, OnIntersect);
    group_value_constructor!(on_key_down, Callback1<KeyDownEvent, bool>, OnKeyDown);
    group_value_constructor!(on_load, Callback<()>, OnLoad);
    group_value_constructor!(on_mouse_down, Callback<bool>, OnMouseDown);
    group_value_constructor!(on_mouse_enter, Callback<()>, OnMouseEnter);
    group_value_constructor!(on_mouse_leave, Callback<()>, OnMouseLeave);
    group_value_constructor!(on_mouse_up, Callback<bool>, OnMouseUp);
    group_value_constructor!(on_submit, Callback<()>, OnSubmit);

    /// Extract [`Computed<String>`] from this [AttrGroupValue] if possible.
    ///
    /// Otherwise (for css and event handlers variants) this gives constant empty string.
    /// For displaying in HTML it's better to use `.embed()` method (which uses this one internally).
    pub fn to_string_or_empty(&self) -> Computed<String> {
        match self {
            Self::AttrValue(AttrValue::Static(val)) => {
                let val = *val;
                Computed::from(move |_| val.to_string())
            }
            Self::AttrValue(AttrValue::String(val)) => {
                let val = val.clone();
                Computed::from(move |_| val.to_string())
            }
            Self::AttrValue(AttrValue::Computed(val)) => val.clone(),
            Self::AttrValue(AttrValue::ComputedOpt(val)) => val.map(|val| val.unwrap_or_default()),
            Self::AttrValue(AttrValue::Value(val)) => val.to_computed(),
            Self::AttrValue(AttrValue::ValueOpt(val)) => {
                val.to_computed().map(|val| val.unwrap_or_default())
            }
            _ => Computed::from(|_| "".to_string()),
        }
    }
}

impl<T: Into<AttrValue>> From<T> for AttrGroupValue {
    fn from(value: T) -> Self {
        Self::AttrValue(value.into())
    }
}

impl EmbedDom for AttrGroupValue {
    fn embed(self) -> DomNode {
        self.to_string_or_empty().embed()
    }
}

impl EmbedDom for &AttrGroupValue {
    fn embed(self) -> DomNode {
        self.to_string_or_empty().embed()
    }
}

/// Can be embedded into [dom!](crate::dom!) macro
///
/// A printable type of your own opts in through [`DomDisplay`](crate::DomDisplay) instead;
/// this trait is for a type which renders real DOM.
///
/// The message carries its own advice because the `dom!` macro calls `EmbedDom::embed(value)`
/// directly, and rustc renders only the message - not the label or the notes - for a failed
/// bound on a trait function called that way.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be embedded in `dom!` - a printable type opts in with `impl vertigo::DomDisplay for YourType {{}}`, a type which renders DOM implements `vertigo::EmbedDom`",
    label = "no `EmbedDom` impl for `{Self}`",
    note = "a value is embedded only through an explicit opt-in - vertigo no longer embeds every `T: ToString`",
    note = "see `vertigo::DomDisplay` for the one-line opt-in, which makes the type an attribute value at the same time"
)]
pub trait EmbedDom {
    fn embed(self) -> DomNode;
}

impl EmbedDom for DomElement {
    fn embed(self) -> DomNode {
        self.into()
    }
}

impl EmbedDom for DomComment {
    fn embed(self) -> DomNode {
        self.into()
    }
}

impl EmbedDom for DomText {
    fn embed(self) -> DomNode {
        self.into()
    }
}

impl EmbedDom for DomNode {
    fn embed(self) -> DomNode {
        self
    }
}

impl EmbedDom for &mut String {
    fn embed(self) -> DomNode {
        DomNode::Text {
            node: DomText::new(self.clone()),
        }
    }
}

/// Anything which opted into [`DomDisplay`](crate::DomDisplay) - the primitives, and any
/// printable type which said so - by value or by reference.
///
/// This blanket used to be over `&T where T: ToString`, with the by-value side an explicit
/// list, so that a downstream type implementing [`Display`](std::fmt::Display) could still
/// provide its own `EmbedDom` and render real DOM instead of a text node. `DomDisplay` is
/// vertigo's own trait rather than a foreign one, so the compiler can see that a type which
/// has not opted in never will, and the by-value blanket no longer collides with such an
/// impl. A type asking for both is a coherence error, which is the right answer: printing and
/// rendering are alternatives.
impl<T: DomDisplay> EmbedDom for T {
    fn embed(self) -> DomNode {
        DomNode::Text {
            node: DomText::new(self.to_string()),
        }
    }
}

impl<T: ToString> EmbedDom for std::rc::Rc<T> {
    fn embed(self) -> DomNode {
        DomNode::Text {
            node: DomText::new((*self).to_string()),
        }
    }
}

/// The string types stay off [`DomDisplay`](crate::DomDisplay) - the marker is shared with
/// attribute values, where each of these has a conversion that beats `to_string` at its own
/// job - so they are embedded by an impl each, references included.
macro_rules! impl_embed_to_string {
    ($($typename:ty),* $(,)?) => {
        $(
            impl EmbedDom for $typename {
                fn embed(self) -> DomNode {
                    DomNode::Text {
                        node: DomText::new(self.to_string()),
                    }
                }
            }
        )*
    };
}

impl_embed_to_string!(
    String,
    &String,
    &str,
    std::borrow::Cow<'_, str>,
    &std::borrow::Cow<'_, str>,
);

/// A reactive value embedded in `dom!` becomes a text node that **patches itself**.
///
/// One `UpdateText` command per change, and the node keeps its id. Wrapping the value in a
/// [`render_value`](crate::Computed::render_value) instead - which is what this used to do -
/// replaced the whole text node on every change: three commands, a new
/// [`DomId`](crate::DomId) each time, and a marker
/// comment left in the document forever. For text that is all cost and no benefit, because
/// the rendered shape is always the same single text node.
///
/// These stay bounded on `ToString` rather than on [`DomDisplay`](crate::DomDisplay), so
/// `Computed<String>`, `Computed<&str>` and `Computed<MyType>` need no opt-in. There is
/// nothing to opt out of: a reactive value can only ever become text here, and a wrapper
/// vertigo owns cannot collide with an `EmbedDom` impl written anywhere else.
impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for &Computed<T> {
    fn embed(self) -> DomNode {
        DomNode::Text {
            node: DomText::new_computed_display(self.clone()),
        }
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for Computed<T> {
    fn embed(self) -> DomNode {
        (&self).embed()
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for Value<T> {
    fn embed(self) -> DomNode {
        self.to_computed().embed()
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for &Value<T> {
    fn embed(self) -> DomNode {
        self.to_computed().embed()
    }
}
