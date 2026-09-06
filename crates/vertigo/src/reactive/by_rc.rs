use std::{fmt, ops::Deref, rc::Rc};

/// An [`Rc`] compared by pointer identity, so a handle whose contents cannot implement
/// [`PartialEq`] can still live in a [`Computed`](crate::Computed).
///
/// [`Computed<T>`](crate::Computed) requires `T: PartialEq` for the equality cutoff. That
/// rules out a shared handle which owns something incomparable — a
/// [`DropResource`](crate::DropResource), a callback, a socket subscription. Such a handle
/// is normally an [`Rc`] handed out by a cache keyed on its inputs, and two handles built
/// from the same inputs are the same allocation, so pointer identity is exactly the
/// comparison wanted.
///
/// ```rust
/// use std::rc::Rc;
/// use vertigo::{ByRc, Computed, DropResource, Value, transaction};
///
/// // A handle which cannot be `PartialEq`: it owns a `DropResource`.
/// struct Subscription {
///     _drop: DropResource,
/// }
///
/// fn subscribe(_channel: &str) -> Rc<Subscription> {
///     Rc::new(Subscription { _drop: DropResource::new(|| {}) })
/// }
///
/// let channel = Value::new("races".to_string());
/// let active: Computed<ByRc<Subscription>> =
///     channel.map(|name| ByRc::new(subscribe(&name)));
///
/// // `Deref` reaches the handle's own API.
/// transaction(|ctx| {
///     let _subscription: ByRc<Subscription> = active.get(ctx);
/// });
/// ```
///
/// Where the handle comes from a cache keyed on its inputs, keeping the *key* in the graph
/// and looking the handle up on read is usually simpler still, and gives the cutoff
/// something meaningful to compare. Reach for `ByRc` when the handle has no such key.
pub struct ByRc<T: ?Sized>(pub Rc<T>);

impl<T> ByRc<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self(value)
    }
}

impl<T: ?Sized> ByRc<T> {
    /// The wrapped handle.
    pub fn inner(&self) -> &Rc<T> {
        &self.0
    }
}

/// Cloning the wrapper clones the [`Rc`], so it never requires `T: Clone`.
impl<T: ?Sized> Clone for ByRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Two wrappers are equal when they point at the same allocation.
impl<T: ?Sized> PartialEq for ByRc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Eq for ByRc<T> {}

impl<T: ?Sized> Deref for ByRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> From<Rc<T>> for ByRc<T> {
    fn from(value: Rc<T>) -> Self {
        Self(value)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for ByRc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ByRc").field(&&*self.0).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Handle(u32);

    #[test]
    fn the_same_allocation_compares_equal() {
        let handle = ByRc::new(Rc::new(Handle(1)));
        assert_eq!(handle.clone(), handle);
    }

    #[test]
    fn an_equal_value_in_a_different_allocation_does_not() {
        let one = ByRc::new(Rc::new(Handle(1)));
        let other = ByRc::new(Rc::new(Handle(1)));
        assert_ne!(one, other);
    }

    #[test]
    fn deref_reaches_the_wrapped_value() {
        let handle = ByRc::new(Rc::new(Handle(7)));
        assert_eq!(handle.0.0, 7);
    }

    #[test]
    fn clone_does_not_require_the_payload_to_be_clone() {
        // `Handle` is not `Clone`; the wrapper still is.
        let handle = ByRc::new(Rc::new(Handle(1)));
        let copy = handle.clone();
        assert!(Rc::ptr_eq(handle.inner(), copy.inner()));
    }
}
