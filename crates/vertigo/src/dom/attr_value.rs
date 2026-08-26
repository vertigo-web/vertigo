use std::rc::Rc;

use crate::{Computed, Css, Value};

/// The text of an attribute, in whichever form costs nothing to keep hold of.
///
/// An element stores the value it last wrote so it can tell a real change from a repeat, and
/// the [`dom!`](crate::dom) macro knows at compile time that `class="col-md-1"` is a literal
/// living in the binary. Boxing that into an `Rc<String>` - which is what every attribute
/// value used to become - costs two heap allocations to hold bytes that were already there.
///
/// Both variants clone by copying a pointer, and compare by their text.
#[derive(Clone, Debug)]
pub enum AttrText {
    Static(&'static str),
    Shared(Rc<String>),
}

impl AttrText {
    pub fn as_str(&self) -> &str {
        match self {
            AttrText::Static(value) => value,
            AttrText::Shared(value) => value.as_str(),
        }
    }
}

impl PartialEq for AttrText {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&'static str> for AttrText {
    fn from(value: &'static str) -> Self {
        AttrText::Static(value)
    }
}

impl From<String> for AttrText {
    fn from(value: String) -> Self {
        AttrText::Shared(Rc::new(value))
    }
}

impl From<Rc<String>> for AttrText {
    fn from(value: Rc<String>) -> Self {
        AttrText::Shared(value)
    }
}

#[derive(Clone)]
pub enum AttrValue {
    /// A literal from the `dom!` macro. Distinguished from [`String`](Self::String) so that
    /// it never has to be copied onto the heap - see [`AttrText`].
    Static(&'static str),
    String(Rc<String>),
    Computed(Computed<String>),
    ComputedOpt(Computed<Option<String>>),
    Value(Value<String>),
    ValueOpt(Value<Option<String>>),
}

impl AttrValue {
    pub fn get(&self, ctx: &crate::Context) -> Option<AttrText> {
        match self {
            AttrValue::Static(s) => Some(AttrText::Static(s)),
            AttrValue::String(s) => Some(AttrText::Shared(s.clone())),
            AttrValue::Computed(c) => Some(AttrText::from(c.get(ctx))),
            AttrValue::ComputedOpt(c) => c.get(ctx).map(AttrText::from),
            AttrValue::Value(v) => Some(AttrText::from(v.get(ctx))),
            AttrValue::ValueOpt(v) => v.get(ctx).map(AttrText::from),
        }
    }

    /// The text this value has right now, if it is known without reading the graph.
    fn as_static_text(&self) -> Option<&str> {
        match self {
            AttrValue::Static(value) => Some(value),
            AttrValue::String(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn combine(classes: Vec<AttrValue>) -> AttrValue {
        let all_static = classes.iter().all(|class| class.as_static_text().is_some());

        if all_static {
            let mut result = Vec::new();
            for class in &classes {
                let Some(text) = class.as_static_text() else {
                    continue;
                };
                let text = text.trim();
                if !text.is_empty() {
                    result.push(text);
                }
            }
            return AttrValue::String(Rc::new(result.join(" ")));
        }

        let computed = crate::Computed::from(move |ctx| {
            let mut result = Vec::new();
            for class in &classes {
                if let Some(s) = class.get(ctx) {
                    let s = s.as_str().trim();
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

impl From<String> for AttrValue {
    fn from(value: String) -> Self {
        AttrValue::String(Rc::new(value))
    }
}

impl From<&&str> for AttrValue {
    fn from(value: &&str) -> Self {
        AttrValue::from(*value)
    }
}

impl From<&str> for AttrValue {
    fn from(value: &str) -> Self {
        AttrValue::String(Rc::new(value.to_string()))
    }
}

impl From<&String> for AttrValue {
    fn from(value: &String) -> Self {
        AttrValue::String(Rc::new(value.clone()))
    }
}

impl From<&&String> for AttrValue {
    fn from(value: &&String) -> Self {
        AttrValue::from(*value)
    }
}

impl From<Rc<String>> for AttrValue {
    fn from(value: Rc<String>) -> Self {
        AttrValue::String(value)
    }
}

impl From<&Rc<String>> for AttrValue {
    fn from(value: &Rc<String>) -> Self {
        AttrValue::String(value.clone())
    }
}

impl From<Rc<str>> for AttrValue {
    fn from(value: Rc<str>) -> Self {
        AttrValue::String(Rc::new(value.to_string()))
    }
}

impl From<&Rc<str>> for AttrValue {
    fn from(value: &Rc<str>) -> Self {
        AttrValue::String(Rc::new(value.to_string()))
    }
}

impl From<std::borrow::Cow<'_, str>> for AttrValue {
    fn from(value: std::borrow::Cow<'_, str>) -> Self {
        AttrValue::String(Rc::new(value.into_owned()))
    }
}

macro_rules! impl_from_display_for_attrvalue {
    ($($typename:ty),* $(,)?) => {
        $(
            impl From<$typename> for AttrValue {
                fn from(value: $typename) -> Self {
                    AttrValue::String(Rc::new(value.to_string()))
                }
            }

            impl From<&$typename> for AttrValue {
                fn from(value: &$typename) -> Self {
                    AttrValue::String(Rc::new(value.to_string()))
                }
            }
        )*
    };
}

impl_from_display_for_attrvalue!(
    bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

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
