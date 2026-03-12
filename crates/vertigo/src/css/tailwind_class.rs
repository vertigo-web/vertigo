use std::{
    borrow::Cow,
    ops::{Add, AddAssign},
};

use crate::{AttrValue, Computed};

/// This represents a tailwind class. Use [tw!](crate::tw!) macro to create one.
#[derive(Clone, PartialEq, Eq)]
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

impl Add<TwClass> for Computed<TwClass> {
    type Output = Computed<TwClass>;

    fn add(self, rhs: TwClass) -> Self::Output {
        self.map(move |left| left.join(&rhs))
    }
}

impl Add<Computed<TwClass>> for Computed<TwClass> {
    type Output = Computed<TwClass>;

    fn add(self, rhs: Computed<TwClass>) -> Self::Output {
        Computed::from({
            let left = self.clone();
            let right = rhs.clone();
            move |ctx| left.get(ctx) + right.get(ctx)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TwClass;
    use crate::{Computed, Value, transaction};

    #[test]
    fn computed_twclass_add_twclass() {
        let value = Value::new(TwClass::from("a"));
        let comp: Computed<TwClass> = value.to_computed();
        let result = comp + TwClass::from("b");

        transaction(|ctx| {
            assert_eq!(result.get(ctx).to_class_value(), "a b");
        });
    }

    #[test]
    fn computed_twclass_add_computed_twclass() {
        let value1 = Value::new(TwClass::from("a"));
        let value2 = Value::new(TwClass::from("b"));

        let comp1: Computed<TwClass> = value1.to_computed();
        let comp2: Computed<TwClass> = value2.to_computed();

        let result = comp1 + comp2;

        transaction(|ctx| {
            assert_eq!(result.get(ctx).to_class_value(), "a b");
        });
    }
}
