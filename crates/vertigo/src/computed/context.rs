use std::marker::PhantomData;

pub struct Context {             //In transaction
    _phantom: PhantomData<()>,
}
impl Context {
    pub(crate) fn new() -> Context {
        Context {
            _phantom: PhantomData
        }
    }
}
