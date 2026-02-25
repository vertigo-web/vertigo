use std::{collections::BTreeMap, rc::Rc};

use crate::{
    Computed, Css, DomComment, DomElement, DomText, DropFileEvent, KeyDownEvent, Value,
    dom::{
        attr_value::AttrValue,
        callback::{Callback, Callback1},
        dom_node::DomNode,
        events::ClickEvent,
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
    OnInput(Rc<Callback1<String, ()>>),
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
    group_value_constructor!(on_input, Callback1<String, ()>, OnInput);
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

impl<T: ToString> EmbedDom for T {
    fn embed(self) -> DomNode {
        DomNode::Text {
            node: DomText::new(self.to_string()),
        }
    }
}

impl<T: ToString + Clone + PartialEq + 'static> EmbedDom for &Computed<T> {
    fn embed(self) -> DomNode {
        self.render_value(|val| DomNode::Text {
            node: DomText::new(val.to_string()),
        })
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
