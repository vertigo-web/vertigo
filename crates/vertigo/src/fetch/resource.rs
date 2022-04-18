use core::ops::{ControlFlow, FromResidual, Try};

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

impl<T> Try for Resource<T> {
    type Output = T;
    type Residual = ResourceError;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Resource::Ready(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<ResourceError, Self::Output> {
        match self {
            Self::Loading => ControlFlow::Break(ResourceError::Loading),
            Self::Error(message) => ControlFlow::Break(ResourceError::Error(message)),
            Self::Ready(value) => ControlFlow::Continue(value),
        }
    }
}

impl<T> FromResidual<ResourceError> for Resource<T> {
    fn from_residual(residual: ResourceError) -> Resource<T> {
        match residual {
            ResourceError::Error(message) => Resource::Error(message),
            ResourceError::Loading => Resource::Loading,
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

/*
Exampl:

fn get1() -> Resource<String> {
    todo!()
}

fn get2() -> Resource<u32> {
    todo!()
}

fn get3() -> Resource<String> {
    let val1: String = get1()?;
    let val2: u32 = get2()?;

    todo!();
}
*/
