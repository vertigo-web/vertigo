use std::{any::Any, rc::Rc};

use crate::{Instant, Resource};

/// What a cache holds, whatever its type.
#[derive(Clone, PartialEq)]
pub enum ApiResponse {
    Uninitialized,
    Data {
        value: Resource<Payload>,
        expiry: Option<Instant>,
    },
}

impl ApiResponse {
    pub fn new(value: Resource<Payload>, expiry: Option<Instant>) -> Self {
        Self::Data { value, expiry }
    }

    pub fn new_loading() -> Self {
        ApiResponse::Data {
            value: Resource::Loading,
            expiry: None,
        }
    }

    pub fn get_value(&self) -> Resource<Payload> {
        match self {
            Self::Uninitialized => Resource::Loading,
            Self::Data { value, expiry: _ } => value.clone(),
        }
    }

    pub fn needs_update(&self) -> bool {
        match self {
            ApiResponse::Uninitialized => true,
            ApiResponse::Data { value: _, expiry } => {
                let Some(expiry) = expiry else {
                    return false;
                };

                expiry.is_expire()
            }
        }
    }
}

/// A value read by a cache, with its type erased.
///
/// The reactive cell a cache keeps it in, the update and the fetch are then the same code
/// whatever the type, rather than a copy of all of them for each type an application fetches.
/// Only the cache of that type puts a value in and takes it out ([`Payload::downcast`]).
#[derive(Clone)]
pub struct Payload {
    value: Rc<dyn Any>,
    /// `PartialEq` of the type inside: a new response equal to the old one is no change, as
    /// with a [`Value`](crate::Value) of that type.
    eq: fn(&dyn Any, &dyn Any) -> bool,
}

impl Payload {
    pub fn new<T: PartialEq + 'static>(value: T) -> Self {
        Self {
            value: Rc::new(value),
            eq: eq_as::<T>,
        }
    }

    /// The value, if it is a `T`.
    pub fn downcast<T: 'static>(self) -> Option<Rc<T>> {
        self.value.downcast().ok()
    }
}

impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        (self.eq)(self.value.as_ref(), other.value.as_ref())
    }
}

fn eq_as<T: PartialEq + 'static>(left: &dyn Any, right: &dyn Any) -> bool {
    match (left.downcast_ref::<T>(), right.downcast_ref::<T>()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::Payload;

    #[test]
    fn payloads_are_equal_when_their_values_are() {
        assert!(Payload::new(5) == Payload::new(5));
        assert!(Payload::new(5) != Payload::new(6));
        assert!(Payload::new(5_i32) != Payload::new(5_i64));
    }

    #[test]
    fn downcast_hands_out_the_value_itself() {
        let payload = Payload::new("text".to_string());
        let first = payload.clone().downcast::<String>();
        let second = payload.clone().downcast::<String>();

        assert_eq!(first.as_deref().map(String::as_str), Some("text"));
        assert!(matches!((first, second), (Some(a), Some(b)) if Rc::ptr_eq(&a, &b)));
        assert!(payload.downcast::<i32>().is_none());
    }
}
