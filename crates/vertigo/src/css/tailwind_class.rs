use std::{
    borrow::Cow,
    ops::{Add, AddAssign},
};

use crate::AttrValue;

/// This represents a tailwind class. Use [tw!](crate::tw!) macro to create one.
#[derive(Clone)]
pub struct TwClass(Cow<'static, str>);

impl TwClass {
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self(value.into())
    }

    pub fn join(&self, value: &TwClass) -> Self {
        let new_str: String = [self.0.as_ref(), value.0.as_ref()].join(" ");
        Self(Cow::Owned(new_str))
    }

    // This is intentionally not implemented as ToString to allow usage only in "tw=" attribute in `dom!` macro.
    pub fn to_class_value(&self) -> String {
        self.0.to_string()
    }
}

impl From<&'static str> for TwClass {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl From<crate::Computed<TwClass>> for crate::AttrValue {
    fn from(value: crate::Computed<TwClass>) -> Self {
        crate::AttrValue::Computed(value.map(|c| c.to_class_value()))
    }
}

impl From<&crate::Computed<TwClass>> for crate::AttrValue {
    fn from(value: &crate::Computed<TwClass>) -> Self {
        crate::AttrValue::Computed(value.clone().map(|c| c.to_class_value()))
    }
}

impl From<TwClass> for AttrValue {
    fn from(value: TwClass) -> Self {
        value.0.to_string().into()
    }
}

impl Add for TwClass {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.join(&rhs)
    }
}

impl AddAssign for TwClass {
    fn add_assign(&mut self, other: Self) {
        *self = self.join(&other);
    }
}
