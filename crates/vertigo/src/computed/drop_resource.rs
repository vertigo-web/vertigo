/// A struct used by [driver](struct.Driver.html) to tidy things up on javascript side after a rust object goes out of scope.
pub struct DropResource {
    drop_fun: Option<Box<dyn FnOnce()>>,
}

impl DropResource {
    pub fn new<F: FnOnce() + 'static>(drop_fun: F) -> DropResource {
        DropResource {
            drop_fun: Some(Box::new(drop_fun)),
        }
    }

    pub fn off(self) {}
}

impl Drop for DropResource {
    fn drop(&mut self) {
        let drop_fun = std::mem::replace(&mut self.drop_fun, None);
        if let Some(drop_fun) = drop_fun {
            drop_fun();
        }
    }
}
