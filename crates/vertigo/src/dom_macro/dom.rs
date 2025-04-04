use std::collections::BTreeMap;

use crate::{
    dom::{attr_value::AttrValue, dom_node::DomNode},
    Callback, Callback1, Computed, Css, DomComment, DomElement, DomText, DropFileEvent,
    KeyDownEvent, Value,
};

/// Type interpreted as component's dynamic attributes groups
///
/// Be careful when using dynamic attributes, key-value type compatibility is checked
/// in runtime (errors logged into JS console) or ignored for `AttrValue` variant.
pub type AttrGroup = BTreeMap<String, AttrGroupValue>;

pub enum AttrGroupValue {
    AttrValue(AttrValue),
    Css(Css),
    HookKeyDown(Callback1<KeyDownEvent, bool>),
    OnBlur(Callback<()>),
    OnChange(Callback1<String, ()>),
    OnClick(Callback<()>),
    OnDropfile(Callback1<DropFileEvent, ()>),
    OnInput(Callback1<String, ()>),
    OnKeyDown(Callback1<KeyDownEvent, bool>),
    OnLoad(Callback<()>),
    OnMouseDown(Callback<bool>),
    OnMouseEnter(Callback<()>),
    OnMouseLeave(Callback<()>),
    OnMouseUp(Callback<bool>),
    OnSubmit(Callback<()>),
    Suspense(fn(bool) -> Css),
}

macro_rules! group_value_constructor {
    ($function:ident, $cb_type:ty, $variant:ident) => {
        pub fn $function(callback: impl Into<$cb_type>) -> Self {
            Self::$variant(callback.into())
        }
    };
}

impl AttrGroupValue {
    pub fn css(css: impl Into<Css>) -> Self {
        Self::Css(css.into())
    }

    group_value_constructor!(hook_key_down, Callback1<KeyDownEvent, bool>, HookKeyDown);
    group_value_constructor!(on_blur, Callback<()>, OnBlur);
    group_value_constructor!(on_change, Callback1<String, ()>, OnChange);
    group_value_constructor!(on_click, Callback<()>, OnClick);
    group_value_constructor!(on_dropfile, Callback1<DropFileEvent, ()>, OnDropfile);
    group_value_constructor!(on_input, Callback1<String, ()>, OnInput);
    group_value_constructor!(on_key_down, Callback1<KeyDownEvent, bool>, OnKeyDown);
    group_value_constructor!(on_load, Callback<()>, OnLoad);
    group_value_constructor!(on_mouse_down, Callback<bool>, OnMouseDown);
    group_value_constructor!(on_mouse_enter, Callback<()>, OnMouseEnter);
    group_value_constructor!(on_mouse_leave, Callback<()>, OnMouseLeave);
    group_value_constructor!(on_mouse_up, Callback<bool>, OnMouseUp);
    group_value_constructor!(on_submit, Callback<()>, OnSubmit);

    pub fn suspense(callback: fn(bool) -> Css) -> Self {
        Self::Suspense(callback)
    }
}

impl<T: Into<AttrValue>> From<T> for AttrGroupValue {
    fn from(value: T) -> Self {
        Self::AttrValue(value.into())
    }
}

impl From<Css> for AttrGroupValue {
    fn from(value: Css) -> Self {
        Self::Css(value)
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
