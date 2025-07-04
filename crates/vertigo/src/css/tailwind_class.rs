use std::{
    borrow::Cow,
    ops::{Add, AddAssign},
};

use crate::AttrValue;

/// This represents a tailwind class. Use [tw!](crate::tw!) macro to create one.
pub struct TwClass<'a>(Cow<'a, str>);

impl<'a> TwClass<'a> {
    pub fn new(value: impl Into<Cow<'a, str>>) -> Self {
        Self(value.into())
    }

    pub fn join<'b>(&self, value: &TwClass<'b>) -> Self {
        let new_str: String = [self.0.as_ref(), value.0.as_ref()].join(" ");
        Self(Cow::Owned(new_str))
    }

    // This is intentionally not implemented as ToString to allow usage only in "tw=" attribute in `dom!` macro.
    pub fn to_class_value(&self) -> String {
        self.0.to_string()
    }
}

impl<'a> From<&'static str> for TwClass<'a> {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl<'a> From<TwClass<'a>> for AttrValue {
    fn from(value: TwClass<'a>) -> Self {
        value.0.to_string().into()
    }
}

impl<'a> Add for TwClass<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.join(&rhs)
    }
}

impl<'a> AddAssign for TwClass<'a> {
    fn add_assign(&mut self, other: Self) {
        *self = self.join(&other);
    }
}
