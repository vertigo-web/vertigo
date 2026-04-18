use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local! {
    static CONTEXT_REGISTRY: RefCell<HashMap<TypeId, Vec<Rc<dyn Any>>>> =
        RefCell::new(HashMap::new());
}

/// RAII guard that pops a context entry when dropped.
pub struct ContextGuard<T: Any + 'static>(PhantomData<T>);

impl<T: Any + 'static> Drop for ContextGuard<T> {
    fn drop(&mut self) {
        CONTEXT_REGISTRY.with(|reg| {
            if let Some(stack) = reg.borrow_mut().get_mut(&TypeId::of::<T>()) {
                stack.pop();
            }
        });
    }
}

/// Push a value onto the context stack for type `T`.
///
/// Returns a [`ContextGuard`] that pops the value when dropped, so the
/// context is automatically cleaned up at the end of the enclosing scope.
///
/// ```rust
/// use std::rc::Rc;
/// use vertigo::{push_context, get_context};
///
/// struct Theme { color: &'static str }
///
/// let _guard = push_context(Rc::new(Theme { color: "blue" }));
/// let theme = get_context::<Theme>().unwrap();
/// assert_eq!(theme.color, "blue");
/// ```
pub fn push_context<T: Any + 'static>(value: Rc<T>) -> ContextGuard<T> {
    CONTEXT_REGISTRY.with(|reg| {
        reg.borrow_mut()
            .entry(TypeId::of::<T>())
            .or_default()
            .push(value);
    });
    ContextGuard(PhantomData)
}

/// Read the most recently pushed context value of type `T`, if any.
///
/// Returns `None` when no context of this type has been pushed.
pub fn get_context<T: Any + 'static>() -> Option<Rc<T>> {
    CONTEXT_REGISTRY.with(|reg| {
        reg.borrow()
            .get(&TypeId::of::<T>())?
            .last()
            .cloned()?
            .downcast::<T>()
            .ok()
    })
}
