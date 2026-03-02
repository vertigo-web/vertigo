use std::rc::Rc;

use crate::{Computed, ToComputed};

/// The state of the resource.
#[derive(Clone, Debug)]
pub enum Resource<T> {
    Loading,
    Ready(T),
    Error(String),
}

pub enum ResourceError {
    Loading,
    Error(String),
}

impl<T> From<ResourceError> for Resource<T> {
    fn from(residual: ResourceError) -> Resource<T> {
        match residual {
            ResourceError::Error(message) => Resource::Error(message),
            ResourceError::Loading => Resource::Loading,
        }
    }
}

impl<T> Resource<T> {
    /// Convert into a `Result` so that `?` can be used inside functions
    /// returning another `Resource`. Use `resource.into_result()?` instead
    /// of the previously-required nightly `Try` impl.
    pub fn into_result(self) -> Result<T, ResourceError> {
        match self {
            Resource::Ready(value) => Ok(value),
            Resource::Loading => Err(ResourceError::Loading),
            Resource::Error(message) => Err(ResourceError::Error(message)),
        }
    }
}

impl<T> Resource<T> {
    pub fn map<K>(self, map: impl Fn(T) -> K) -> Resource<K> {
        match self {
            Resource::Loading => Resource::Loading,
            Resource::Ready(data) => Resource::Ready(map(data)),
            Resource::Error(err) => Resource::Error(err),
        }
    }

    pub fn ref_map<K>(&self, map: impl Fn(&T) -> K) -> Resource<K> {
        match self {
            Resource::Loading => Resource::Loading,
            Resource::Ready(data) => Resource::Ready(map(data)),
            Resource::Error(err) => Resource::Error(err.clone()),
        }
    }
}

impl<T: Clone> Resource<T> {
    #[must_use]
    pub fn ref_clone(&self) -> Self {
        match self {
            Resource::Loading => Resource::Loading,
            Resource::Ready(data) => Resource::Ready(data.clone()),
            Resource::Error(error) => Resource::Error(error.clone()),
        }
    }
}

impl<T: PartialEq> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Resource::Loading, Resource::Loading) => true,
            (Resource::Error(message1), Resource::Error(message2)) => message1.eq(message2),
            (Resource::Ready(val1), Resource::Ready(val2)) => val1.eq(val2),
            _ => false,
        }
    }
}

impl<T: Clone + 'static> ToComputed<Resource<Rc<T>>> for Resource<T> {
    fn to_computed(&self) -> crate::Computed<Resource<Rc<T>>> {
        Computed::from({
            let myself = self.clone();
            move |_| myself.clone().map(|item| Rc::new(item))
        })
    }
}

impl<T: Clone + 'static> ToComputed<Resource<Rc<T>>> for Computed<Resource<T>> {
    fn to_computed(&self) -> crate::Computed<Resource<Rc<T>>> {
        self.map(|res| res.map(|item| Rc::new(item)))
    }
}
