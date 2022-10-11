/// Clone only reference
///
/// Allows `dom!` macro to acquire component's parameters by move if passed as bare T,
/// or clone it if passed by reference.
pub trait CloneOrNoOp<T> {
    fn clone_or_no_op(self) -> T;
}

impl<T> CloneOrNoOp<T> for T {
    fn clone_or_no_op(self) -> T {
        self
    }
}

impl<'a, T: Clone> CloneOrNoOp<T> for &'a T {
    fn clone_or_no_op(self) -> T {
        self.clone()
    }
}

pub fn clone_if_ref<T, K: CloneOrNoOp<T>>(data: K) -> T {
    data.clone_or_no_op()
}
