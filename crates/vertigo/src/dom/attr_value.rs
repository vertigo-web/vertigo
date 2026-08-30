use std::rc::Rc;

use crate::{Computed, Css, DomDisplay, Value};

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

/// The bound on every attribute value - what [`DomElement::attr`](crate::DomElement::attr)
/// and the `dom!` macro accept.
///
/// Vertigo implements it exactly once, as a blanket over `Into<AttrValue>`; it exists so that
/// the compiler has somewhere to hang an explanation. The conversions that blanket picks up
/// are the `From` impls below plus whatever opts in through [`DomDisplay`], and a type
/// outside both fails a `From` bound whose error names a hundred impls and none of the ones
/// you want.
///
/// A crate of its own *may* implement this trait directly for its own type - coherence
/// permits it, since only that crate could ever give the type a [`DomDisplay`] impl. But
/// [`DomDisplay`] is one line rather than a function body, it also reaches `&T`,
/// [`Computed<T>`](crate::Computed) and [`Value<T>`](crate::Value), and it renders the same
/// type as a text node in [`dom!`](crate::dom) too.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as an attribute value",
    label = "no conversion from `{Self}` into `AttrValue`",
    note = "a value becomes an attribute through an explicit conversion - vertigo no longer converts every `T: ToString`",
    note = "a printable type of your own opts in with one line: `impl vertigo::DomDisplay for YourType {{}}`, which covers the value, references to it, its `Computed`/`Value` wrappers, and embedding it as text",
    note = "if it comes from another crate, pass `value.to_string()` or wrap it in a newtype of your own"
)]
pub trait IntoAttrValue {
    fn into_attr_value(self) -> AttrValue;
}

/// Without this, a missing conversion is reported as the `From` obligation *inside* this
/// impl - a hundred `From` impls and no advice - rather than as the bound the call site
/// actually wrote.
#[diagnostic::do_not_recommend]
impl<T: Into<AttrValue>> IntoAttrValue for T {
    fn into_attr_value(self) -> AttrValue {
        self.into()
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

/// Covers `&T` too, via the reference impl on [`DomDisplay`] itself.
impl<T: DomDisplay> From<T> for AttrValue {
    fn from(value: T) -> Self {
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

/// A reactive attribute of any type which opted into [`DomDisplay`], printed on each read.
///
/// The `String` and `Option<String>` cases above are the same conversions without the
/// `to_string` - they stay separate impls so that the common case does not map through a
/// second node.
macro_rules! impl_attr_display_computed {
    ($typename:ty, |$var:ident| $body:expr) => {
        impl<T: DomDisplay + Clone + PartialEq + 'static> From<$typename> for AttrValue {
            fn from($var: $typename) -> Self {
                AttrValue::Computed($body)
            }
        }
    };
}

impl_attr_display_computed!(Computed<T>, |v| v.map(|v| v.to_string()));
impl_attr_display_computed!(&Computed<T>, |v| v.map(|v| v.to_string()));
impl_attr_display_computed!(Value<T>, |v| v.to_computed().map(|v| v.to_string()));
impl_attr_display_computed!(&Value<T>, |v| v.to_computed().map(|v| v.to_string()));

macro_rules! impl_attr_display_computed_opt {
    ($typename:ty, |$var:ident| $body:expr) => {
        impl<T: DomDisplay + Clone + PartialEq + 'static> From<$typename> for AttrValue {
            fn from($var: $typename) -> Self {
                AttrValue::ComputedOpt($body)
            }
        }
    };
}

impl_attr_display_computed_opt!(Computed<Option<T>>, |v| v.map(|v| v.map(|v| v.to_string())));
impl_attr_display_computed_opt!(&Computed<Option<T>>, |v| v
    .map(|v| v.map(|v| v.to_string())));
impl_attr_display_computed_opt!(Value<Option<T>>, |v| v
    .to_computed()
    .map(|v| v.map(|v| v.to_string())));
impl_attr_display_computed_opt!(&Value<Option<T>>, |v| v
    .to_computed()
    .map(|v| v.map(|v| v.to_string())));

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
