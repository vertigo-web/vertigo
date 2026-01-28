use std::any::Any;

/// A struct used by [driver](struct.Driver.html) to tidy things up on javascript side after a rust object goes out of scope.
pub enum DropResource {
    Fun(Option<Box<dyn FnOnce()>>),
    Struct(Box<dyn Any>),
}

impl DropResource {
    pub fn new<F: FnOnce() + 'static>(drop_fun: F) -> DropResource {
        DropResource::Fun(Some(Box::new(drop_fun)))
    }

    pub fn from_struct(inst: impl Any) -> DropResource {
        DropResource::Struct(Box::new(inst))
    }

    pub fn off(self) {}
}

impl PartialEq for DropResource {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Drop for DropResource {
    fn drop(&mut self) {
        match self {
            Self::Fun(inner) => {
                let drop_fun = std::mem::take(inner);

                if let Some(drop_fun) = drop_fun {
                    drop_fun();
                }
            }
            Self::Struct(_) => {}
        }
    }
}
