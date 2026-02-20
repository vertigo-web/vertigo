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

impl AttrValue {
    pub fn get(&self, ctx: &crate::computed::context::Context) -> Option<Rc<String>> {
        match self {
            AttrValue::String(s) => Some(s.clone()),
            AttrValue::Computed(c) => Some(Rc::new(c.get(ctx))),
            AttrValue::ComputedOpt(c) => c.get(ctx).map(Rc::new),
            AttrValue::Value(v) => Some(Rc::new(v.get(ctx))),
            AttrValue::ValueOpt(v) => v.get(ctx).map(Rc::new),
        }
    }

    pub fn combine(classes: Vec<AttrValue>) -> AttrValue {
        let mut all_static = true;
        for class in &classes {
            if !matches!(class, AttrValue::String(_)) {
                all_static = false;
                break;
            }
        }

        if all_static {
            let mut result = Vec::new();
            for class in classes {
                if let AttrValue::String(s) = class {
                    let s = s.trim();
                    if !s.is_empty() {
                        result.push(s.to_string());
                    }
                }
            }
            return AttrValue::String(Rc::new(result.join(" ")));
        }

        let computed = crate::Computed::from(move |ctx| {
            let mut result = Vec::new();
            for class in &classes {
                if let Some(s) = class.get(ctx) {
                    let s = s.trim();
                    if !s.is_empty() {
                        result.push(s.to_string());
                    }
                }
            }
            result.join(" ")
        });

        AttrValue::Computed(computed)
    }
}

impl<K: ToString> From<K> for AttrValue {
    fn from(value: K) -> Self {
        AttrValue::String(Rc::new(value.to_string()))
    }
}

macro_rules! impl_from_computed_for_attrvalue {
    ($typename:ty, $variant:ident, |$var:ident| $body:expr) => {
        impl From<$typename> for AttrValue {
            fn from($var: $typename) -> Self {
                AttrValue::$variant($body)
            }
        }
    };
    ($typename: ty, $variant: ident) => {
        impl_from_computed_for_attrvalue!($typename, $variant, |v| v);
    };
}

impl_from_computed_for_attrvalue!(Computed<String>, Computed);
impl_from_computed_for_attrvalue!(Computed<Option<String>>, ComputedOpt);
impl_from_computed_for_attrvalue!(&Computed<String>, Computed, |v| v.clone());
impl_from_computed_for_attrvalue!(&Computed<Option<String>>, ComputedOpt, |v| v.clone());

impl_from_computed_for_attrvalue!(Value<String>, Value);
impl_from_computed_for_attrvalue!(Value<Option<String>>, ValueOpt);
impl_from_computed_for_attrvalue!(&Value<String>, Value, |v| v.clone());
impl_from_computed_for_attrvalue!(&Value<Option<String>>, ValueOpt, |v| v.clone());

macro_rules! impl_stringable_into_computed_for_attrvalue {
    ($typename:ty) => {
        impl_from_computed_for_attrvalue!(Computed<$typename>, Computed, |v| v
            .map(|v| v.to_string()));
        impl_from_computed_for_attrvalue!(&Computed<$typename>, Computed, |v| v
            .map(|v| v.to_string()));
        impl_from_computed_for_attrvalue!(Value<$typename>, Computed, |v| v
            .to_computed()
            .map(|v| v.to_string()));
        impl_from_computed_for_attrvalue!(&Value<$typename>, Computed, |v| v
            .to_computed()
            .map(|v| v.to_string()));
    };
}

impl_stringable_into_computed_for_attrvalue!(i8);
impl_stringable_into_computed_for_attrvalue!(i16);
impl_stringable_into_computed_for_attrvalue!(i32);
impl_stringable_into_computed_for_attrvalue!(i64);
impl_stringable_into_computed_for_attrvalue!(isize);

impl_stringable_into_computed_for_attrvalue!(u8);
impl_stringable_into_computed_for_attrvalue!(u16);
impl_stringable_into_computed_for_attrvalue!(u32);
impl_stringable_into_computed_for_attrvalue!(u64);
impl_stringable_into_computed_for_attrvalue!(usize);

impl_stringable_into_computed_for_attrvalue!(f32);
impl_stringable_into_computed_for_attrvalue!(f64);

impl_stringable_into_computed_for_attrvalue!(char);

impl_stringable_into_computed_for_attrvalue!(bool);

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
