use crate::utils::EqBox;

/// A struct used by [driver](struct.Driver.html) to tidy things up on javascript side after a rust object goes out of scope.
#[derive(PartialEq, Debug)]
pub struct DropResource {
    drop_fun: Option<EqBox<Box<dyn FnOnce()>>>,
}

impl DropResource {
    pub fn new<F: FnOnce() + 'static>(drop_fun: F) -> DropResource {
        DropResource {
            drop_fun: Some(EqBox::new(Box::new(drop_fun))),
        }
    }

    pub fn off(self) {}
}

impl Drop for DropResource {
    fn drop(&mut self) {
        let drop_fun = std::mem::replace(&mut self.drop_fun, None);
        if let Some(drop_fun) = drop_fun {
            let drop_fun = drop_fun.into_inner();
            drop_fun();
        }
    }
}
