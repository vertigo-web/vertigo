use std::rc::Rc;

use crate::{Computed, Css, Value};

#[derive(Clone)]
pub enum AttrValue {
    String(Rc<String>),
    Computed(Computed<String>),
    ComputedOpt(Computed<Option<String>>),
    Value(Value<String>),
    ValueOpt(Value<Option<String>>),
}

impl<K: ToString> From<K> for AttrValue {
    fn from(value: K) -> Self {
        AttrValue::String(Rc::new(value.to_string()))
    }
}

impl From<Computed<String>> for AttrValue {
    fn from(value: Computed<String>) -> Self {
        AttrValue::Computed(value)
    }
}

impl From<Computed<Option<String>>> for AttrValue {
    fn from(value: Computed<Option<String>>) -> Self {
        AttrValue::ComputedOpt(value)
    }
}

impl From<&Computed<Option<String>>> for AttrValue {
    fn from(value: &Computed<Option<String>>) -> Self {
        AttrValue::ComputedOpt(value.clone())
    }
}

impl From<Value<String>> for AttrValue {
    fn from(value: Value<String>) -> Self {
        AttrValue::Value(value)
    }
}

impl From<Value<Option<String>>> for AttrValue {
    fn from(value: Value<Option<String>>) -> Self {
        AttrValue::ValueOpt(value)
    }
}

impl From<&Value<String>> for AttrValue {
    fn from(value: &Value<String>) -> Self {
        AttrValue::Value(value.clone())
    }
}

impl From<&Value<Option<String>>> for AttrValue {
    fn from(value: &Value<Option<String>>) -> Self {
        AttrValue::ValueOpt(value.clone())
    }
}

pub enum CssAttrValue {
    Css(Css),
    Computed(Computed<Css>),
}

impl From<Css> for CssAttrValue {
    fn from(value: Css) -> Self {
        CssAttrValue::Css(value)
    }
}

impl From<Computed<Css>> for CssAttrValue {
    fn from(value: Computed<Css>) -> Self {
        CssAttrValue::Computed(value)
    }
}

impl From<&Css> for CssAttrValue {
    fn from(value: &Css) -> Self {
        CssAttrValue::Css(value.clone())
    }
}
