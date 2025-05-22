use std::{borrow::Cow, ops::{Add, AddAssign, Deref}};

use crate::AttrValue;

/// This represents a tailwind class. Use [tw!] macro to create one.
pub struct TwClass<'a>(Cow<'a, str>);

impl<'a> Deref for TwClass<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TwClass<'a> {
    pub fn new(value: impl Into<Cow<'a, str>>) -> Self {
        Self(value.into())
    }

    pub fn join<'b>(&self, value: &TwClass<'b>) -> Self {
        let new_str: String = [(*self).as_ref(), (*value).as_ref()].join(" ");
        Self(Cow::Owned(new_str))
    }
}

impl<'a> From<TwClass<'a>> for AttrValue {
    fn from(value: TwClass<'a>) -> Self {
        (*value).to_string().into()
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
