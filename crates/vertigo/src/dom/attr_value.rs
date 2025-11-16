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
